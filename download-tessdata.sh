#!/bin/bash

mkdir -p tessdata
cd tessdata

if [ ! -f eng.traineddata ]; then
    echo "Downloading English language data for Tesseract..."
    curl -L -o eng.traineddata https://github.com/tesseract-ocr/tessdata_fast/raw/main/eng.traineddata
    echo "Download complete!"
else
    echo "English language data already exists."
fi

cd ..
echo "Tessdata is ready in ./tessdata/"
