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
    count = 2
    network {
      port "http" {}
    }

    service {
      name = "bookclub-prod"
      port = "http"
      // check {
      //   type     = "http"
      //   port     = "http"
      //   path     = "/_health"
      //   timeout  = "1s"
      //   interval = "10s"
      //   check_restart {
      //     limit = 5
      //     grace = "30s"
      //   }
      // }
      tags = [
        "traefik.enable=true",
        "traefik.http.routers.bookclub.rule=Host(`bookclub.tomhoward.codes`)"
        "traefik.http.routers.traefik.entrypoints=websecure",
        "traefik.http.routers.tls.certresovler=letsencrypt",
        "traefik.http.routers.traefik.middlewares=authelia@docker",
        "traefik.http.services.traefik.loadbalancer.server.port=${NOMAD_PORT_http}"
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

      # The "identity" block instructs Nomad to expose the task's workload
      # identity token as an environment variable and in the file
      # secrets/nomad_token.
      identity {
        env  = true
        file = true
      }

      resources {
        cpu    = 500 # 500 MHz
        memory = 256 # 256MB
      }


      # The "template" block instructs Nomad to manage a template, such as
      # a configuration file or script. This template can optionally pull data
      # from Consul or Vault to populate runtime configuration data.
      #
      # For more information and examples on the "template" block, please see
      # the online documentation at:
      #
      #     https://developer.hashicorp.com/nomad/docs/job-specification/template
      #
      # template {
      #   data          = "---\nkey: {{ key \"service/my-key\" }}"
      #   destination   = "local/file.yml"
      #   change_mode   = "signal"
      #   change_signal = "SIGHUP"
      # }

      # The "template" block can also be used to create environment variables
      # for tasks that prefer those to config files. The task will be restarted
      # when data pulled from Consul or Vault changes.
      #
      # template {
      #   data        = "KEY={{ key \"service/my-key\" }}"
      #   destination = "local/file.env"
      #   env         = true
      # }

      # vault {
      #   policies      = ["cdn", "frontend"]
      #   change_mode   = "signal"
      #   change_signal = "SIGHUP"
      # }

      kill_timeout = "30s"
    }
  }
}
