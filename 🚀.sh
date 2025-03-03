#!/bin/bash

yarn env &&
    echo "😴 Sleeping 1 seconds" &&
    sleep 1 &&
    yarn test-cargo &&
    echo "😴 Sleeping 1 seconds" &&
    sleep 1 &&
    osascript -e 'tell application "Terminal" to do script "cd '"$PWD"' && yarn validator-new"' &&
    echo "😴 Sleeping 1 seconds" &&
    sleep 1 &&
    yarn prep &&
    echo "😴 Sleeping 1 seconds" &&
    sleep 1 &&
    yarn build &&
    echo "😴 Sleeping 1 seconds" &&
    sleep 1 &&
    yarn deploy &&
    echo "😴 Sleeping 5 seconds" &&
    sleep 5 &&
    yarn setup &&
    echo "😴 Sleeping 5 seconds" &&
    sleep 5 &&
    yarn initialise &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn admin &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn new &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn deposit &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn equalise &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn withdraw-bulk &&
    echo "😴 Sleeping 10 seconds" &&
    sleep 10 &&
    yarn withdraw-single
