#!/bin/bash

# Default commit message
MESSAGE=${1:-"Update codebase"}

git add .
git commit -m "$MESSAGE"
git push
