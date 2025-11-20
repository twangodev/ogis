# ogis: Open Graph Images as a Service

[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/twangodev/ogis/rust.yml?logo=rust)](https://github.com/twangodev/ogis/actions)
[![Docker Pulls](https://img.shields.io/docker/pulls/twango/ogis?logo=docker)](https://hub.docker.com/r/twango/ogis)
[![Docker Image Size](https://img.shields.io/docker/image-size/twango/ogis/latest?logo=docker)](https://hub.docker.com/r/twango/ogis)
[![Codecov](https://img.shields.io/codecov/c/github/twangodev/ogis)](https://codecov.io/gh/twangodev/ogis)
[![License](https://img.shields.io/github/license/twangodev/ogis)](https://github.com/twangodev/ogis/blob/main/LICENSE)


Generating [open graph images](https://ogp.me/) is a lot more work than it should be.
ogis abstracts away the complexity into a fast, simple to use platform to generate beautiful images.

## Features

All generation endpoints support the following features:

- **Dynamic Text**: Customize `title`, `subtitle`, and `description` text
- **Custom Logo**: Add your own `logo`, as well as an `image`. This should be a publicly accessible URL.
- **Font Selection**: Choose from a variety of `font` options, with support for international characters
- **Template Choice**: Choose built-in `template` designs, make your own, and customize `colors` as needed
- **Authentication**: Secure your image generation with HMAC `signature` validation

All of this is available via a simple HTTP API, which you can configure using query parameters.

```http
GET https://img.ogis.dev/?title=OpenGraph&description=MadeEasy
```

## Examples

ogis comes with a variety of built-in templates to get you started quickly. 
Check out the full supported templates at [ogis.dev/templates](https://ogis.dev/templates).

<table>
  <tr>
    <td align="center">
      <img src="https://img.ogis.dev/?template=twilight&title=Dark%20%26%20Bold&subtitle=Default%20Template&description=Perfect%20for%20tech%20and%20development%20content&logo=https://ogis.dev/logo-light.png" alt="Twilight" width="100%"/>
      <br/><strong>twilight</strong>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=daybreak&title=Light%20%26%20Fresh&subtitle=Morning%20Vibes&description=Clean%20and%20professional%20design&logo=https://ogis.dev/logo-dark.png" alt="Daybreak" width="100%"/>
      <br/><strong>daybreak</strong>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=minimal&title=Simple%20%26%20Clean&subtitle=Minimalist%20Design&description=Ultra-clean%20centered%20layout" alt="Minimal" width="100%"/>
      <br/><strong>minimal</strong>
    </td>
  </tr>
  <tr>
    <td align="center">
      <img src="https://img.ogis.dev/?template=stripe&title=Enterprise%20Ready&subtitle=Production%20Grade&description=Scalable%20image%20generation%20for%20your%20platform&logo=https://ogis.dev/logo-dark.png" alt="Stripe" width="100%"/>
      <br/><strong>stripe</strong>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=gradient&title=Make%20It%20Pop!&subtitle=Endless%20Possibilities&description=Beautiful%20images%20in%20milliseconds&logo=https://ogis.dev/logo-light.png" alt="Gradient" width="100%"/>
      <br/><strong>gradient</strong>
    </td>
    <td align="center">
      <img src="https://img.ogis.dev/?template=hero&title=Dramatic%20Impact&subtitle=Full%20Background&description=Photo-heavy%20content%20with%20overlay&logo=https://ogis.dev/logo-light.png" alt="Hero" width="100%"/>
      <br/><strong>hero</strong>
    </td>
  </tr>
  <tr>
    <td align="center">
      <img src="https://img.ogis.dev/?template=modern&title=Contemporary%20Style&subtitle=Geometric%20Shapes&description=Subtle%20and%20sophisticated%20design&logo=https://ogis.dev/logo-dark.png" alt="Modern" width="100%"/>
      <br/><strong>modern</strong>
    </td>
    <td align="center">Add your own!</td>
    <td align="center">Or submit a PR :)</td>
  </tr>
</table>

## Deploy

[![Deploy on Railway](https://img.shields.io/badge/Deploy%20on-Railway-0B0D0E?style=for-the-badge&logo=railway&logoColor=white)](https://railway.com/deploy/Ax4OH3?referralCode=rXZ78U&utm_medium=integration&utm_source=template&utm_campaign=generic)
[![Deploy to Google Cloud](https://img.shields.io/badge/Deploy%20to-Google%20Cloud-4285F4?style=for-the-badge&logo=googlecloud&logoColor=white)](https://deploy.cloud.run/?git_repo=https://github.com/twangodev/ogis)

Deploy ogis with a single command:

```bash
docker run -d -p 3000:3000 twango/ogis:latest
```

Or use Docker Compose for development:

```bash
docker compose up -d
```

> [!TIP]
> The Docker image is self-contained and works with any container orchestration platform (Kubernetes, Docker Swarm, etc.). It runs on port 3000 and includes a `/health` endpoint for readiness/liveness probes.