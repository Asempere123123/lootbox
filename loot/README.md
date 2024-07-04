# Lootbox
Lootbox creates isolated Python installations to easily manage multiple Python versions on your computer. It allows you to create projects with its CLI and add and manage dependencies for them. It was created by a student, and for serious usage, you should probably check out Poetry instead. It is also written in Rust, so it's automatically faster and better than Poetry. ;)
## Usage
You can create and manage project dependencies and install Python versions right now. It also provides a simple utility for bundling the project if you want to get a simpler version with requirements.txt. If I have time, I might even add the option to use it to publish packages to PyPI (adding the functionality of "pyproject.toml", "setup.py", etc., directly to Lootbox).
### Installation
```
pip install py-lootbox
```
### Install Python Version
This works differently on Linux and Windows to avoid the weird behaviors of the Python installer. On Windows, the version is installed using NuGet, and on Linux (and maybe macOS, though I haven't tested it), it builds the Python version from source (Python tarball), so the installation can be quite slow.
```
loot install {version_to_install}
loot install 3.10.0
```
### Create project
```
loot new {name} {python_version}
loot new test 3.10.0

cd {name}
cd test
```
### Run project
```
loot run
```
### Add dependency
```
loot add {package_name}
loot add bs4
```
### Bundle
```
loot bundle
```
### Run command inside venv
In some cases you might want to run a command inside the projects venv. This is common if one of your dependencies provides a cli.
```
loot exec {command_to_run}
loot exec alembic -h
```