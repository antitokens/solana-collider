#!/bin/bash

FILES=$1

# Print the files to confirm input
echo "Files to propagate: $FILES"

# Check if on main
if git rev-parse --abbrev-ref HEAD | grep -q 'main'; then
    echo "✓ On main"

    # Define an array of branches to propagate changes
    branches=("backup" "devnet" "localnet" "prod")

    # Commit and push changes on main
    git add $FILES &&
        git commit -S -m "Propagate $FILES" &&
        git push

    # Iterate through each branch and propagate changes
    for branch in "${branches[@]}"; do
        git checkout $branch &&
            git checkout main -- $FILES &&
            git commit -S -m "Propagate $FILES" &&
            git push
    done

    # Return to main
    git checkout main
else
    echo "✕ Not on main"
fi
