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

  group "server" {
    count = 1

    network {
      port "http" {}
    }

    service {
      name = "bookclub-prod"
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
        "traefik.enable=true",
        "traefik.http.routers.bookclub.rule=Host(`bookclub.tomhoward.codes`)",
        "traefik.http.routers.bookclub.entrypoints=websecure",
        "traefik.http.routers.bookclub.tls.certresolver=letsencrypt",
        "traefik.http.routers.bookclub.middlewares=authelia@docker",
        "traefik.http.services.bookclub.loadbalancer.server.port=${NOMAD_PORT_http}"
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

          {{ range service "bookclubcache-prod"}}
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

          {{ range service "bookclubcache-prod"}}
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
      name     = "bookclubcache-prod"
      tags     = ["global", "cache"]
      port     = "db"
      provider = "nomad"

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
