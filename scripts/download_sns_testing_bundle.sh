#!/bin/bash

# get the latest comment from https://github.com/dfinity/ic/commits/master/
IC_COMMIT=e4c691f1ade69eb55b06feadaea5fa9eb334716b

# download the sns testing bundle
OS=linux
curl --fail \
    -L "https://download.dfinity.systems/ic/${IC_COMMIT}/binaries/x86_64-${OS}/sns_testing_bundle.tar.gz" \
    -o src/test_helper/sns_testing_bundle/sns_testing_bundle.tar.gz

# extract the sns testing bundle
tar -xvf src/test_helper/sns_testing_bundle/sns_testing_bundle.tar.gz -C src/test_helper/sns_testing_bundle
rm src/test_helper/sns_testing_bundle/sns_testing_bundle.tar.gz