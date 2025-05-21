#!/bin/bash

# Compress the nns_state directory into nns_state.tar.gz
tar -zcvf src/test_helper/nns_state.tar.gz -C src/test_helper nns_state

# If compression is successful, remove the extracted directory
if [ $? -eq 0 ]; then
    rm -rf src/test_helper/nns_state
fi
