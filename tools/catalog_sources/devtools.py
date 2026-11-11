"""Single-binary developer tools, compilers and servers published as Windows ZIP releases on GitHub."""
from .runtimes import _releases


def uv():
    return _releases('astral-sh/uv','uv','uv',
        'Python package and project manager. It can also install Python versions of its own.',
        lambda _:'uv-x86_64-pc-windows-msvc.zip',lambda _:'uv.exe','Development tools')


def pnpm():
    return _releases('pnpm/pnpm','pnpm','pnpm',
        'Fast, disk-efficient package manager for Node.js projects. Standalone build; install Node.js separately.',
        lambda _:'pnpm-win32-x64.zip',lambda _:'pnpm.exe','Development tools')


def gh():
    return _releases('cli/cli','gh','GitHub CLI',
        'Work with GitHub pull requests, issues and releases from the terminal. Sign in with gh auth login.',
        lambda tag:f'gh_{tag.removeprefix("v")}_windows_amd64.zip',lambda _:'bin/gh.exe','Development tools')


def hugo():
    return _releases('gohugoio/hugo','hugo','Hugo',
        'Static site generator. Extended edition with Sass support; run hugo server inside a site folder.',
        lambda tag:f'hugo_extended_{tag.removeprefix("v")}_windows-amd64.zip',lambda _:'hugo.exe','Development tools')


def typst():
    return _releases('typst/typst','typst','Typst',
        'Markup-based typesetting system that compiles documents to PDF.',
        lambda _:'typst-x86_64-pc-windows-msvc.zip',lambda filename:filename.removesuffix('.zip')+'/typst.exe','Development tools')


def pocketbase():
    return _releases('pocketbase/pocketbase','pocketbase','PocketBase',
        'Backend in one file: SQLite database, auth, file storage and admin UI. Start with the serve argument.',
        lambda tag:f'pocketbase_{tag.removeprefix("v")}_windows_amd64.zip',lambda _:'pocketbase.exe','Development tools')


def gleam():
    return _releases('gleam-lang/gleam','gleam','Gleam',
        'Gleam compiler and build tool. Running on the BEAM needs Erlang/OTP installed separately; the JavaScript target needs Node.js, Deno or Bun.',
        lambda tag:f'gleam-{tag}-x86_64-pc-windows-msvc.zip',lambda _:'gleam.exe','SDKs & compilers')


def traefik():
    return _releases('traefik/traefik','traefik','Traefik',
        'Reverse proxy and load balancer. Provide a static configuration file or arguments before starting.',
        lambda tag:f'traefik_{tag}_windows_amd64.zip',lambda _:'traefik.exe','Web servers')


def frankenphp():
    return _releases('php/frankenphp','frankenphp','FrankenPHP',
        'PHP application server built on Caddy, with PHP bundled. Start with php-server or a Caddyfile.',
        lambda _:'frankenphp-windows-x86_64.zip',lambda _:'frankenphp.exe','Web servers')
