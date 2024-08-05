// TODO: Migrate to Nomad Services from Consul Services so I can use the function to just get one.
job "bookclub" {
  datacenters = ["*"]
  type        = "service"

  update {
    max_parallel      = 1
    min_healthy_time  = "10s"
    healthy_deadline  = "3m"
    progress_deadline = "10m"
    auto_revert       = false
    canary            = 0
  }

  migrate {
    max_parallel     = 1
    health_check     = "checks"
    min_healthy_time = "10s"
    healthy_deadline = "5m"
  }

  group "cloudflare" {
    count = 1

    task "cloudflare" {
      driver = "docker"
      config {
        image        = "cloudflare/cloudflared:latest"
        force_pull   = true
        network_mode = "host"
        command      = "tunnel"
        args         = ["--config", "/secrets/tunnel.yaml", "run"]
      }

      resources {
        cpu    = 100
        memory = 64
      }

      vault {
        policies = ["bookclub-prod"]
      }

      template {
        data        = <<EOH
{{- with secret "kv/inkwellcollective/cloudflare" }}
{
  "AccountTag":"{{ .Data.AccountTag }}",
  "TunnelSecret":"{{ .Data.TunnelSecret }}",
  "TunnelID":"{{ .Data.TunnelID }}"
}
{{- end }}
          EOH
        destination = "secrets/credentials.json"
        env         = false
      }

      # TODO: HA Proxy to scale up the number of servers running. Cloudflared can only route to 1 thing.
      template {
        data        = <<EOH
{{- with secret "kv/inkwellcollective/cloudflare" }}
tunnel: {{ .Data.TunnelID }} 
{{- end }}
credentials-file: /secrets/credentials.json

ingress:
  {{- range $i, $s := service "inkwellcollective-traefik"}}
  {{- if eq $i 0}}
  - hostname: inkwellcollective.org
    service: https://{{ .Address }}:{{ .Port }}
    originRequest:
      noTLSVerify: true
  - hostname: *.inkwellcollective.org
    service: https://{{ .Address }}:{{ .Port }}
    originRequest:
      noTLSVerify: true
  {{- end }}
  {{- end }}
  - service: http_status:404
        EOH
        destination = "secrets/tunnel.yaml"
        env         = false
      }
    }
  }

  group "traefik" {
    count = 1

    ephemeral_disk {
      migrate = true
      size    = 300
      sticky  = true
    }

    network {
      port "https" {}
      port "dashboard" {}
    }

    service {
      name = "inkwellcollective-traefik"
      port = "https"
      check {
        type     = "http"
        port     = "https"
        path     = "/ping"
        interval = "10s"
        timeout  = "2s"
      }
    }

    service {
      name = "inkwellcollective-traefikdashboard"
      port = "dashboard"
      check {
        type     = "http"
        port     = "dashboard"
        path     = "/ping"
        interval = "10s"
        timeout  = "2s"
      }
      tags = [
        "inkwellcollectivetraefik.enable=true",
        "inkwellcollectivetraefik.http.routers.traefik.rule=Host(`traefik.inkwellcollective.org`)",
        "inkwellcollectivetraefik.http.routers.traefik.entrypoints=websecure",
        "inkwellcollectivetraefik.http.routers.traefik.tls.certresolver=letsencrypt"
      ]
    }

    task "traefik" {
      driver = "docker"
      config {
        image        = "traefik:v2.9"
        network_mode = "host"
        args = [
          "--api.insecure",
          "--log.level=DEBUG",

          "--entrypoints.traefik.address=:${NOMAD_PORT_dashboard}",
          "--entrypoints.websecure.address=:${NOMAD_PORT_https}",
          "--entrypoints.websecure.forwardedHeaders.insecure=true",
          "--entrypoints.websecure.http.middlewares=inkwellcollective-authelia@consulcatalog",

          "--providers.consulCatalog=true",
          "--providers.consulCatalog.endpoint.address=http://127.0.0.1:8500",
          "--providers.consulCatalog.prefix=inkwellcollectivetraefik",

          "--certificatesresolvers.letsencrypt.acme.dnsChallenge=true",
          "--certificatesresolvers.letsencrypt.acme.dnsChallenge.provider=cloudflare",
          "--certificatesresolvers.letsencrypt.acme.email=info@tomhoward.codes",
          "--certificatesresolvers.letsencrypt.acme.storage=/alloc/data/acme.json",
        ]
      }

      resources {
        cpu    = 100
        memory = 64
      }

      vault {
        policies = ["bookclub-prod"]
      }

      template {
        data        = <<EOH
        {{ with secret "kv/cloudflare" }}
        CF_API_EMAIL="{{ .Data.CF_API_EMAIL }}"
        CF_DNS_API_TOKEN="{{ .Data.CF_DNS_API_TOKEN }}"
        CF_ZONE_API_TOKEN="{{ .Data.CF_ZONE_API_TOKEN }}"
        {{ end }}
        EOH
        destination = "secrets/cloudflare.env"
        env         = true
      }
    }
  }

  group "ldap" {
    count = 1

    network {
      port "ldap" {}
      port "http" {}
    }

    service {
      name = "inkwellcollective-ldap"
      port = "ldap"
      check {
        type     = "tcp"
        port     = "ldap"
        interval = "10s"
        timeout  = "2s"
      }
    }
    service {
      name = "inkwellcollective-ldaphttp"
      port = "http"
      check {
        type     = "http"
        port     = "http"
        path     = "/"
        interval = "10s"
        timeout  = "2s"
      }
      tags = [
        "inkwellcollectivetraefik.enable=true",
        "inkwellcollectivetraefik.http.routers.ldap.rule=Host(`ldap.inkwellcollective.org`)",
        "inkwellcollectivetraefik.http.routers.ldap.entrypoints=websecure",
        "inkwellcollectivetraefik.http.routers.ldap.tls.certresolver=letsencrypt"
      ]
    }

    task "ldap" {
      driver = "docker"
      config {
        image        = "nitnelave/lldap:stable"
        ports        = ["ldap", "http"]
        network_mode = "host"
      }

      resources {
        cpu    = 100
        memory = 64
      }

      env {
        TZ                                        = "America/New_York"
        LLDAP_LDAP_HOST                           = "0.0.0.0"
        LLDAP_LDAP_PORT                           = "${NOMAD_PORT_ldap}"
        LLDAP_HTTP_HOST                           = "0.0.0.0"
        LLDAP_HTTP_PORT                           = "${NOMAD_PORT_http}"
        LLDAP_HTTP_URL                            = "https://ldap.inkwellcollective.org"
        LLDAP_LDAP_BASE_DN                        = "dc=inkwellcollective,dc=org"
        LLDAP_LDAP_USER_DN                        = "admin"
        LLDAP_LDAP_USER_EMAIL                     = "info@tomhoward.codes"
        LLDAP_SMTP_OPTIONS__ENABLE_PASSWORD_RESET = "false"
      }

      vault {
        policies = ["bookclub-prod"]
      }

      template {
        data        = <<EOH
          {{ with secret "kv/inkwellcollective/lldap" }}
          LLDAP_JWT_SECRET="{{ .Data.jwt_secret }}"
          LLDAP_LDAP_USER_PASS="{{ .Data.ldap_user_pass }}"
          LLDAP_KEY_SEED="{{ .Data.key_seed }}"
          {{ end }}
          {{ with secret "database/creds/bookclub-prod" }}
          LLDAP_DATABASE_URL="postgres://{{.Data.username}}:{{.Data.password}}@127.0.0.1:5432/inkwellcollective_lldap"
          {{ end }}
        EOH
        destination = "secrets/service.env"
        env         = true
      }
    }
  }

  group "authelia" {
    count = 1

    network {
      port "http" {}
    }

    service {
      name = "inkwellcollective-authelia"
      port = "http"
      check {
        type     = "http"
        port     = "http"
        path     = "/_ping"
        timeout  = "1s"
        interval = "10s"
        check_restart {
          limit = 5
          grace = "30s"
        }
      }
      tags = [
        "inkwellcollectivetraefik.enable=true",
        "inkwellcollectivetraefik.http.routers.authelia.rule=Host(`auth.inkwellcollective.org`)",
        "inkwellcollectivetraefik.http.routers.authelia.entrypoints=websecure",
        "inkwellcollectivetraefik.http.routers.authelia.tls.certresolver=letsencrypt",
        # Middleware Registration.
        "inkwellcollectivetraefik.http.middlewares.authelia-inkwellcollective.forwardAuth.address=http://{{ env $NOMAD_ADDR_http }}/api/verify?rd=https%3A%2F%2Fauth.inkwellcollective.org",
        "inkwellcollectivetraefik.http.middlewares.authelia-inkwellcollective.forwardAuth.trustForwardHeader=true",
        "inkwellcollectivetraefik.http.middlewares.authelia-inkwellcollective.forwardAuth.authResponseHeaders=Remote-User,Remote-Groups,Remote-Name,Remote-Email",
      ]
    }

    task "authelia" {
      driver = "docker"
      config {
        image        = "authelia/authelia:4"
        ports        = ["http"]
        network_mode = "host"
        mount {
          type   = "bind"
          source = "secrets"
          target = "/config"
        }
      }

      resources {
        cpu    = 100
        memory = 64
      }

      vault {
        policies = ["bookclub-prod"]
      }

      template {
        data        = <<EOH
{{- with secret "kv/inkwellcollective/authelia" }}
jwt_secret: {{ .Data.jwt_secret }}
{{- end }}
theme: dark
default_2fa_method: ""
server:
  host: {{ env "NOMAD_IP_http" }}
  port: {{ env "NOMAD_PORT_http" }}
  enable_pprof: false
  enable_expvars: false
  disable_healthcheck: false

log:
  level: info

telemetry:
  metrics:
    enabled: false
  
totp:
  disable: false
  issuer: authelia.com
  period: 60
  skew: 1
  secret_size: 32

webauthn:
  disable: false
  timeout: 60s
  display_name: Authelia
  attestation_conveyance_preference: 'indirect'
  user_verification: preferred

ntp:
  address: "time.cloudflare.com:123"
  version: 4
  max_desync: 3s
  disable_startup_check: false
  disable_failure: false

authentication_backend:
  password_reset:
    disable: false
    custom_url: ""
  refresh_interval: 1m
  ldap:
    implementation: custom
    {{- range service "inkwellcollective-ldap" }}
    address: "ldap://{{ .Address }}:{{ .Port }}"
    {{- end }}
    start_tls: false
    base_dn: dc=inkwellcollective,dc=org
    username_attribute: uid
    additional_users_dn: ou=people
    users_filter: (&(|({username_attribute}={input})({mail_attribute}={input}))(objectClass=person))
    additional_groups_dn: ou=groups
    groups_filter: (member={dn})
    group_name_attribute: cn
    mail_attribute: mail
    display_name_attribute: displayName
    user: uid=admin,ou=people,dc=inkwellcollective,dc=org
    {{- with secret "kv/inkwellcollective/lldap" }}
    password: "{{ .Data.ldap_user_pass }}"
    {{- end }}
password_policy:
  standard:
    enabled: false
  zxcvbn:
    enabled: true
    min_score: 3

access_control:
  default_policy: two_factor
  rules:
    - domain: "www.inkwellcollective.org"
      policy: one_factor

session:
  name: auth_session
  domain: inkwellcollective.org
  same_site: lax
  {{- with secret "kv/inkwellcollective/authelia" }}
  secret: {{ .Data.session_secret }}
  {{- end }}
  expiration: 6h
  inactivity: 2h
  remember_me_duration: 14d

  redis:
    {{- range service  "inkwellcollective-redis|any" }}
    host: {{ .Address }}
    port: {{ .Port }}
    {{- end }}
    database_index: 1
  
regulation:
  max_retries: 5
  find_time: 2m
  ban_time: 60m

storage:
  {{- with secret "kv/inkwellcollective/authelia" }}
  encryption_key: {{ .Data.encryption_key }}
  {{- end }}
  postgres:
    host: 127.0.0.1
    port: 5432
    database: inkwellcollective_authelia
    schema: public
    {{- with secret "database/creds/bookclub-prod" }}
    username: {{ .Data.username }}
    password: {{ .Data.password }}
    {{- end }}

notifier:
  disable_startup_check: true
  smtp:
    host: smtp.postmark.app
    port: 25
    {{- with secret "kv/inkwellcollective/postmark" }}
    username: {{ .Data.username }}
    password: {{ .Data.password }}
    {{- end }}
    sender: "Authelia <auth@inkwellcollective.org>"
    disable_require_tls: false
EOH
        destination = "secrets/configuration.yml"
      }
    }
  }



  group "server" {
    count = 1

    network {
      port "http" {}
    }

    service {
      name = "inkwellcollective-server"
      port = "http"
      check {
        type     = "http"
        port     = "http"
        path     = "/_ping"
        timeout  = "1s"
        interval = "10s"
        check_restart {
          limit = 5
          grace = "30s"
        }
      }
      tags = [
        "inkwellcollectivetraefik.enable=true",
        "inkwellcollectivetraefik.http.routers.bookclub.rule=Host(`www.inkwellcollective.org`)",
        "inkwellcollectivetraefik.http.routers.bookclub.entrypoints=websecure",
        "inkwellcollectivetraefik.http.routers.bookclub.tls.certresolver=letsencrypt",
        "inkwellcollectivetraefik.http.routers.bookclub.middlewares=authelia-inkwellcollective@consulcatalog",
        "inkwellcollectivetraefik.http.services.bookclub.loadbalancer.server.port=${NOMAD_PORT_http}"
      ]
    }

    task "bookclub" {
      driver = "docker"
      config {
        image        = "thoward27/bookclub:main"
        force_pull   = true
        network_mode = "host"
        ports        = ["http"]
        command      = "start"
      }

      resources {
        cpu    = 100
        memory = 64
      }

      vault {
        policies = ["bookclub-prod"]
      }

      env {
        PORT     = "${NOMAD_PORT_http}"
        HOST     = "https://bookclub.tomhoward.codes"
        LOCO_ENV = "production"
      }

      template {
        data        = <<EOH
          {{ with secret "database/creds/bookclub-prod" }}
          DATABASE_URL="postgres://{{.Data.username}}:{{.Data.password}}@127.0.0.1:5432/bookclub_production"
          {{ end }}

          {{ range service "inkwellcollective-redis"}}
          REDIS_URL="redis://{{.Address}}:{{.Port}}"
          {{ end }}
        EOH
        destination = "secrets/service.env"
        env         = true
      }
    }
  }

  group "worker" {
    count = 1

    task "worker" {
      driver = "docker"
      config {
        image        = "thoward27/bookclub:main"
        force_pull   = true
        network_mode = "host"
        command      = "start"
        args         = ["--worker"]
      }

      resources {
        cpu    = 100
        memory = 64
      }

      vault {
        policies = ["bookclub-prod"]
      }

      env {
        LOCO_ENV = "production"
      }

      template {
        data        = <<EOH
          {{ with secret "database/creds/bookclub-prod" }}
          DATABASE_URL="postgres://{{.Data.username}}:{{.Data.password}}@127.0.0.1:5432/bookclub_production"
          {{ end }}

          {{ range service "inkwellcollective-redis"}}
          REDIS_URL="redis://{{.Address}}:{{.Port}}"
          {{ end }}
        EOH
        destination = "secrets/service.env"
        env         = true
      }
    }
  }

  group "cache" {
    count = 1

    network {
      port "db" {
        to = 6379
      }
    }

    service {
      name = "inkwellcollective-redis"
      port = "db"
      check {
        name     = "alive"
        type     = "tcp"
        interval = "10s"
        timeout  = "2s"
      }
    }

    restart {
      attempts = 2
      interval = "30m"
      delay    = "15s"
      mode     = "fail"
    }

    ephemeral_disk {
      sticky  = true
      migrate = true
      size    = 300
    }

    task "redis" {
      driver = "docker"
      config {
        image          = "redis:7"
        ports          = ["db"]
        auth_soft_fail = true
      }

      identity {
        env  = true
        file = true
      }

      resources {
        cpu    = 100 # 500 MHz
        memory = 64  # 256MB
      }

      kill_timeout = "30s"
    }
  }
}
