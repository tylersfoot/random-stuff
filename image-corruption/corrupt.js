var originalImage = new Image();
var corruptedImage = new Image();
var corruptType = 'random'; // random, offset
var corruptStrength = 1000; // amount of bytes to change
var originalHexData = '';


function imageToBMPHex(image) {
    // create an off-screen canvas
    let canvas = document.createElement('canvas');
    let ctx = canvas.getContext('2d');

    // set canvas dimensions to image dimensions
    canvas.width = image.width;
    canvas.height = image.height;

    // draw the image onto the canvas
    ctx.drawImage(image, 0, 0);

    // get the raw pixel data
    let imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);

    // convert the image data to BMP format
    let width = imageData.width;
    let height = imageData.height;
    let pixels = imageData.data;
    let pad = (4 - ((width * 3) % 4)) % 4; // padding for 4-byte alignment

    // BMP header
    let fileSize = 54 + (width * 3 + pad) * height;
    let buffer = new ArrayBuffer(fileSize);
    let datav = new DataView(buffer);

    let pos = 0;

    // BMP header
    datav.setUint8(pos++, 0x42); // B
    datav.setUint8(pos++, 0x4D); // M
    datav.setUint32(pos, fileSize, true); pos += 4; // file size
    datav.setUint16(pos, 0, true); pos += 2; // reserved
    datav.setUint16(pos, 0, true); pos += 2; // reserved
    datav.setUint32(pos, 54, true); pos += 4; // offset to pixel data

    // DIB header
    datav.setUint32(pos, 40, true); pos += 4; // DIB header size
    datav.setInt32(pos, width, true); pos += 4; // width
    datav.setInt32(pos, -height, true); pos += 4; // height (negative to indicate top-down bitmap)
    datav.setUint16(pos, 1, true); pos += 2; // planes
    datav.setUint16(pos, 24, true); pos += 2; // bits per pixel (RGB)
    datav.setUint32(pos, 0, true); pos += 4; // compression (none)
    datav.setUint32(pos, fileSize - 54, true); pos += 4; // image size
    datav.setInt32(pos, 2835, true); pos += 4; // horizontal resolution (pixel per meter)
    datav.setInt32(pos, 2835, true); pos += 4; // vertical resolution (pixel per meter)
    datav.setUint32(pos, 0, true); pos += 4; // colors in color table (none)
    datav.setUint32(pos, 0, true); pos += 4; // important color count (all)

    // pixel data
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            let i = (y * width + x) * 4;
            let r = pixels[i];
            let g = pixels[i + 1];
            let b = pixels[i + 2];
            datav.setUint8(pos++, b);
            datav.setUint8(pos++, g);
            datav.setUint8(pos++, r);
        }
        pos += pad; // padding
    }

    // convert the BMP data to a hex string
    let hex = '';
    let byteArray = new Uint8Array(buffer);
    for (let i = 0; i < byteArray.byteLength; i++) {
        let byteHex = byteArray[i].toString(16).padStart(2, '0');
        hex += byteHex;
    }
    let hexData = hex;
    
    return hexData;
}

function png2bmp(pngArrayBuffer) {
    const pngBlob = new Blob([pngArrayBuffer], { type: 'image/png' });
    const img = new Image();
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');

    return new Promise((resolve) => {
        const url = URL.createObjectURL(pngBlob);
        img.onload = () => {
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);

            const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
            const width = imageData.width;
            const height = imageData.height;
            const pixels = imageData.data;
            const pad = (4 - ((width * 3) % 4)) % 4; // Padding for 4-byte alignment

            // BMP header
            const fileSize = 54 + (width * 3 + pad) * height;
            const buffer = new ArrayBuffer(fileSize);
            const datav = new DataView(buffer);

            let pos = 0;

            // BMP Header
            datav.setUint8(pos++, 0x42); // B
            datav.setUint8(pos++, 0x4D); // M
            datav.setUint32(pos, fileSize, true); pos += 4; // file size
            datav.setUint16(pos, 0, true); pos += 2; // reserved
            datav.setUint16(pos, 0, true); pos += 2; // reserved
            datav.setUint32(pos, 54, true); pos += 4; // offset to pixel data

            // DIB Header
            datav.setUint32(pos, 40, true); pos += 4; // DIB header size
            datav.setInt32(pos, width, true); pos += 4; // width
            datav.setInt32(pos, -height, true); pos += 4; // height (negative to indicate top-down bitmap)
            datav.setUint16(pos, 1, true); pos += 2; // planes
            datav.setUint16(pos, 24, true); pos += 2; // bits per pixel (RGB)
            datav.setUint32(pos, 0, true); pos += 4; // compression (none)
            datav.setUint32(pos, fileSize - 54, true); pos += 4; // image size
            datav.setInt32(pos, 2835, true); pos += 4; // horizontal resolution (pixel per meter)
            datav.setInt32(pos, 2835, true); pos += 4; // vertical resolution (pixel per meter)
            datav.setUint32(pos, 0, true); pos += 4; // colors in color table (none)
            datav.setUint32(pos, 0, true); pos += 4; // important color count (all)

            // Pixel data
            for (let y = 0; y < height; y++) {
                for (let x = 0; x < width; x++) {
                    const i = (y * width + x) * 4;
                    const r = pixels[i];
                    const g = pixels[i + 1];
                    const b = pixels[i + 2];
                    datav.setUint8(pos++, b);
                    datav.setUint8(pos++, g);
                    datav.setUint8(pos++, r);
                }
                pos += pad; // padding
            }

            URL.revokeObjectURL(url);
            resolve(buffer);
        };
        img.src = url;
    });
}


function bmp2png(bmpArrayBuffer) {
    const bmpBlob = new Blob([bmpArrayBuffer], { type: 'image/bmp' });
    const img = new Image();
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');

    return new Promise((resolve) => {
        const url = URL.createObjectURL(bmpBlob);
        img.onload = () => {
            canvas.width = img.width;
            canvas.height = img.height;
            ctx.drawImage(img, 0, 0);

            canvas.toBlob((blob) => {
                const reader = new FileReader();
                reader.onload = function(event) {
                    resolve(event.target.result);
                };
                reader.readAsArrayBuffer(blob);
            }, 'image/png');

            URL.revokeObjectURL(url);
        };
        img.src = url;
    });
}

function buf2hex(buffer) {
    const byteArray = new Uint8Array(buffer);
    const hexParts = [];
    for (let i = 0; i < byteArray.length; i++) {
        const hex = byteArray[i].toString(16).padStart(2, '0');
        hexParts.push(hex);
    }
    return hexParts.join('');
}

function hex2buf(hex) {
    const length = hex.length / 2;
    const buffer = new ArrayBuffer(length);
    const view = new Uint8Array(buffer);
    for (let i = 0; i < length; i++) {
        view[i] = parseInt(hex.substr(i * 2, 2), 16);
    }
    return buffer;
}

function buf2img(buffer) {
    const blob = new Blob([buffer], {type: 'image/png'});
    const url = URL.createObjectURL(blob);
    const img = new Image();
    img.src = url;
    return img;
}


function displayImage(type, image) {
    // type: original, corrupted
    // img: Image object
    if (type === 'original') {
        var imageElement = document.getElementById('originalImageDisplay');
    } else {
        var imageElement = document.getElementById('corruptedImageDisplay');
    }
    imageElement.src = image.src;
}

async function corruptImage() {
    try {
        // hexData = imageToBMPHex(originalImage);
        console.log(originalHexData);

        let bmpArrayBuffer = await png2bmp(hex2buf(originalHexData)); // Wait for the BMP ArrayBuffer to be created
        let temphex = buf2hex(bmpArrayBuffer);
        // console.log(temphex);

        if (corruptType === 'random') {
            // loops through the image data and corrupts a random byte
            for (let i = 0; i < corruptStrength; i++) {
                let index = Math.floor(Math.random() * (temphex.length / 2)) * 2;
                let newByte = Math.floor(Math.random() * 256).toString(16).padStart(2, '0');
                let temphex2 = temphex.substring(0, index) + newByte + temphex.substring(index + 2);
                temphex = temphex2;
            }
        }
        // console.log(temphex);
        const corruptedPngArrayBuffer = await bmp2png(hex2buf(temphex));
        temphex = buf2hex(corruptedPngArrayBuffer);
        // console.log(temphex);
        corruptedImage = buf2img(hex2buf(temphex));
        displayImage("corrupted", corruptedImage);
        console.log("Image corrupted successfully.");
    } catch (error) {
        console.error("Error corrupting image:", error);
    }
}

function onFileSelected(event) {
    var selectedFile = event.target.files[0];
    if (selectedFile) {
        var reader = new FileReader();

        // load the file as a data URL for the image display
        reader.onload = function(event) {
            originalImage.src = event.target.result;
            
            // when the image is fully loaded, display it
            originalImage.onload = function() {
                displayImage('original', originalImage);
                console.log("Image loaded successfully.");
            };
        };
        
        // read the file as an ArrayBuffer to get the hex representation
        var hexReader = new FileReader();
        hexReader.onload = function(event) {
            originalHexData = buf2hex(event.target.result);
        };
        
        // initiate both reads
        reader.readAsDataURL(selectedFile);
        hexReader.readAsArrayBuffer(selectedFile);
    }
}

document.addEventListener('DOMContentLoaded', (event) => {
    // gets run when the page fully loads (html/css)
});

