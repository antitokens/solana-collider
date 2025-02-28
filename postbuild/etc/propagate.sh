FILES=$1

# Check if on main
if git rev-parse --abbrev-ref HEAD | grep -q 'main'; then
    echo "✓ On main"
    git add $FILES && git commit -S -m '$FILES' && git push && git checkout backup && git checkout main -- $FILES && git commit -S -m '$FILES' && git push && git checkout devnet && git checkout main -- $FILES && git commit -S -m '$FILES' && git push && git checkout localnet && git checkout main -- $FILES && git commit -S -m '$FILES' && git push && git checkout prod && git checkout main -- $FILES && git commit -S -m '$FILES' && git push && git checkout main
else
    echo "✕ Not on main"
fi
