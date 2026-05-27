#!/bin/bash


clear


read -p "Enter Commit Message " message


git add .

git commit -m "$message"

git push


read -n 1 -s -r -p "Done"
