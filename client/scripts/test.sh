#!/bin/sh

mkdir ~/Dropbox/dreamcaller/client/test_output
/Applications/Unity/Hub/Editor/6000.0.34f1/Unity.app/Contents/MacOS/Unity -runTests -batchmode -projectPath ~/Dropbox/dreamcaller/client -testResults ~/Dropbox/dreamcaller/client/test_output/results.xml -testPlatform PlayMode -logFile ~/Dropbox/dreamcaller/client/test_output/logs.txt
