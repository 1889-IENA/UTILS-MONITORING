#!/bin/bash


clear


read -p "Enter Commit Message " message

read -p "Enter Branch Name " branch


git checkout --orphan temp_branch


git add -A

git commit -am "$message"

git branch -D $branch

git branch -m $branch

git push -f origin $branch


read -n 1 -s -r -p "Done"
