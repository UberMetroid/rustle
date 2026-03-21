# Self Hosted, self contained [Wordle](https://www.powerlanguage.co.uk/wordle/) clone

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/UberMetroid/rust-wordle/Legacy%20Docker%20CI.yml?label=Legacy%20Docker%20Build)
[![GitHub last commit](https://img.shields.io/github/last-commit/UberMetroid/rust-wordle)](https://github.com/UberMetroid/rust-wordle)

Image is based on Nginx stable alpine, and all the content is local to the container.

# Container Screenshot
![image](https://user-images.githubusercontent.com/4349962/152651710-32fc8be9-b63a-47b3-b1f3-ec7baf0e34f8.png)

# Configuration

```yaml
version: "2.4"

services:

  wordle:
    image: ghcr.io/ubermetroid/rust-wordle/legacy:latest
    container_name: Wordle
    ports:
      - 80:80
```

# Tags
| Tag | Description |
| :----: | --- |
| latest | Legacy version |
