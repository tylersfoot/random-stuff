const vertexCode = `
attribute vec4 aPosition;
varying vec4 vPosition;

void main() {
  vPosition = aPosition;
  gl_Position = vPosition;
}
`;
const fragmentCode = `
precision mediump float;

uniform float animationTime;
uniform vec2 parallax; // x and y movement
uniform vec3 colors[16];
uniform vec2 canvasSize;
uniform vec2 screenSize;
uniform float scale; // 1.0 is normal

varying vec4 vPosition;

const float PI2 = 6.283185307179586476925286766559;

uniform sampler2D endTexture;

mat2 mat2_rotate_z(float rad) {
  // returns the rotation matrix for a vec2 around the z-axis
  rad = mod(rad, PI2);
  return mat2(
    cos(rad), -sin(rad),
    sin(rad), cos(rad)
  );
}

vec4 end_portal_layer(float layer, vec4 proj) {
  mat4 scale_translate = mat4(
    0.5, 0.0, 0.0, 17.0 / layer + 0.25,
    0.0, 0.5, 0.0, (2.0 + layer / 1.5) * (animationTime * 1.5) + 0.25,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
  );

  // scale and resize to prevent stretching
  proj.xy *= canvasSize.xy * (0.00070 / scale);

  // resize based on screen size (roughly), so that 4k and 1080p look the same size
  if (screenSize.x / screenSize.y > 1.0) {
    proj.xy *= 1080.0 / screenSize.x;
  } else {
    proj.xy *= 1920.0 / screenSize.y;
  }

  // x and y translation for parallax (mouse or scrolling)
  // 759375000.0 = (15 ^ 5) / 0.001
  proj.xy -= parallax * (layer*layer*layer*layer*layer) / 759375000.0;

  // scale the layer
  mat2 scale = mat2((4.5 - layer / 4.0) * 2.0);
  // rotate the layer
  mat2 rotate = mat2_rotate_z(radians((layer * layer * 4321.0 + layer * 9.0) * 2.0));
  
  // scale, rotation, and two translations
  mat4 megamat = mat4(rotate * scale) * scale_translate;
  
  // apply to projection, mod to loop image
  proj.xyz = mod((proj * megamat).xyz, 1.0);

  return proj;
}

void main() {
  vec3 color = vec3(0.0);
  for (int i = 0; i < 15; i++) {
    color += texture2DProj(endTexture, end_portal_layer(float(i+1), vPosition)).rgb * colors[int(mod(float(i), 16.0))];
  }
  gl_FragColor = vec4(color, 1.0);
}
`;

var gl;
var shaderCanvas;
var shaderDiv;
var shaderParallaxUniformLocation;
var shaderAnimationTimeUniformLocation;
var shaderCanvasSizeUniformLocation;
var shaderScreenSizeUniformLocation;
var shaderScaleUniformLocation;

var shaderParallaxCurrent = [0, 0];
var shaderParallaxTarget = [0, 0];
var shaderLerpSpeed = 0.02;
var shaderLastTime = 0;
var shaderFrame = 0;
var shaderTime = 0;
var shaderScale = 1;

// wallpaper engine properties
var fps = 60; // 1 - 240
var mouseReactive = true; // true or false
var mouseSpeed = 3; // 0 - 10
var idleSpeed = 1; // 0 - 10
var shaderScale = 1; // 0.1 - 10

window.wallpaperPropertyListener = {
  applyUserProperties: function(properties) {
    if (properties.mousereactive) {
      mouseReactive = properties.mousereactive.value;
    }
    if (properties.fps) {
      fps = properties.fps.value;
      // CHANGE FPS
    }
    if (properties.mousespeed) {
      mouseSpeed = properties.mousespeed.value;
    }
    if (properties.idlespeed) {
      idleSpeed = properties.idlespeed.value;
    }
    if (properties.scale) {
      shaderScale = properties.scale.value;
    }
  },
};

function lerp(start, end, amt) {
  return (1-amt)*start+amt*end
}

document.addEventListener('mousemove', (event) => {
  if (!mouseReactive) return;
  shaderParallaxTarget[0] += event.movementX;
  shaderParallaxTarget[1] += event.movementY;
});

function createShader(gl, type, source) {
  var shader = gl.createShader(type);
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  var success = gl.getShaderParameter(shader, gl.COMPILE_STATUS);
  if (success) {
    return shader;
  }
 
  console.log(gl.getShaderInfoLog(shader));
  gl.deleteShader(shader);
}

function createProgram(gl, vertexShader, fragmentShader) {
  var program = gl.createProgram();
  gl.attachShader(program, vertexShader);
  gl.attachShader(program, fragmentShader);
  gl.linkProgram(program);
  var success = gl.getProgramParameter(program, gl.LINK_STATUS);
  if (success) {
    return program;
  }
 
  console.log(gl.getProgramInfoLog(program));
  gl.deleteProgram(program);
}

document.addEventListener('DOMContentLoaded', () => {
  shaderCanvas = document.querySelector('#end-portal-canvas');
  shaderDiv = document.querySelector('#end-portal');
  gl = shaderCanvas.getContext('webgl');

  if (gl) {
    // load shaders from html and create the shaders
    var vertexShader = createShader(gl, gl.VERTEX_SHADER, vertexCode);
    var fragmentShader = createShader(gl, gl.FRAGMENT_SHADER, fragmentCode);

    shaderProgram = createProgram(gl, vertexShader, fragmentShader);

    // tell it to use our program (pair of shaders)
    gl.useProgram(shaderProgram);

    shaderPositionAttributeLocation = gl.getAttribLocation(shaderProgram, "aPosition"); // look up the location of the attribute
    var shaderPositionBuffer = gl.createBuffer(); // create a buffer
    gl.bindBuffer(gl.ARRAY_BUFFER, shaderPositionBuffer); // bind it to ARRAY_BUFFER (think of it as ARRAY_BUFFER = positionBuffer)
    var shaderPositions = [
      -1.0, -1.0, // bottom left
       1.0, -1.0, // bottom right
      -1.0,  1.0, // top left
       1.0, -1.0, // bottom right
      -1.0,  1.0, // top left
       1.0,  1.0  // top right
    ];
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(shaderPositions), gl.STATIC_DRAW);

    var shaderEndTexture = new Image();
    shaderEndTexture.onload = function() {
      gl.activeTexture(gl.TEXTURE0);
      var texture = gl.createTexture();
      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, shaderEndTexture);
      
      // set texture parameters
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);

      var shaderEndTextureLocation = gl.getUniformLocation(shaderProgram, "endTexture");
      gl.uniform1i(shaderEndTextureLocation, 0);

      webglUtils.resizeCanvasToDisplaySize(gl.canvas);
      // tells WebGL the -1 +1 clip space maps to 0 <-> gl.canvas.width for x and 0 <-> gl.canvas.height for y
      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    
      // clear the canvas
      gl.clearColor(0, 0, 0, 0);
      gl.clear(gl.COLOR_BUFFER_BIT);
    
      // tell the attribute how to get data out of positionBuffer (ARRAY_BUFFER)
      var size = 2;          // 2 components per iteration
      var type = gl.FLOAT;   // the data is 32bit floats
      var normalize = false; // don't normalize the data
      var stride = 0;        // 0 = move forward size * sizeof(type) each iteration to get the next position
      var offset = 0;        // start at the beginning of the buffer

      gl.enableVertexAttribArray(shaderPositionAttributeLocation); // turn on the attribute
      gl.bindBuffer(gl.ARRAY_BUFFER, shaderPositionBuffer); // bind the position buffer
      gl.vertexAttribPointer(shaderPositionAttributeLocation, size, type, normalize, stride, offset)

      shaderParallaxUniformLocation = gl.getUniformLocation(shaderProgram, "parallax");
      shaderAnimationTimeUniformLocation = gl.getUniformLocation(shaderProgram, "animationTime");
      shaderCanvasSizeUniformLocation = gl.getUniformLocation(shaderProgram, "canvasSize");
      shaderScreenSizeUniformLocation = gl.getUniformLocation(shaderProgram, "screenSize");
      shaderScaleUniformLocation = gl.getUniformLocation(shaderProgram, "scale");
      var shaderColorsUniformLocation = gl.getUniformLocation(shaderProgram, "colors");

      const shaderColors = [0.022087, 0.098399, 0.110818,
        0.011892, 0.095924, 0.089485,
        0.027636, 0.101689, 0.100326,
        0.046564, 0.109883, 0.114838,
        0.064901, 0.117696, 0.097189,
        0.063761, 0.086895, 0.123646,
        0.084817, 0.111994, 0.166380,
        0.097489, 0.154120, 0.091064,
        0.106152, 0.131144, 0.195191,
        0.097721, 0.110188, 0.187229,
        0.133516, 0.138278, 0.148582,
        0.070006, 0.243332, 0.235792,
        0.196766, 0.142899, 0.214696,
        0.047281, 0.315338, 0.321970,
        0.204675, 0.390010, 0.302066,
        0.080955, 0.314821, 0.661491]
      gl.uniform3fv(shaderColorsUniformLocation, shaderColors);
  
      // start rendering loop
      requestAnimationFrame(render);
    };

    shaderEndTexture.src = 'end_portal_256.png';
  } else {
    console.error('WebGL not supported');
  }
});

function render(shaderCurrentTime) {
  var shaderTimeSinceLastFrame = shaderCurrentTime - shaderLastTime;

  if (shaderTimeSinceLastFrame > (1000 / fps)) {
    shaderLastTime = shaderCurrentTime - (shaderTimeSinceLastFrame % (1000 / fps));

    if (mouseReactive) {
      var shaderParallaxAmount = shaderLerpSpeed * shaderTimeSinceLastFrame;
      shaderParallaxAmount /= shaderParallaxAmount + 5;
      shaderParallaxCurrent[0] = lerp(shaderParallaxCurrent[0], shaderParallaxTarget[0], shaderParallaxAmount);
      shaderParallaxCurrent[1] = lerp(shaderParallaxCurrent[1], shaderParallaxTarget[1], shaderParallaxAmount);
      gl.uniform2f(shaderParallaxUniformLocation, (shaderParallaxCurrent[0] * mouseSpeed) / 10, (-shaderParallaxCurrent[1] * mouseSpeed) / 10); // vec2 output
    }

    // use Date.now() for global sync
    gl.uniform1f(shaderAnimationTimeUniformLocation, ((Date.now() * idleSpeed) % 1200000) / 1200000);

    if (shaderDiv.clientWidth !== gl.canvas.width || shaderDiv.clientHeight !== gl.canvas.height) {
      gl.canvas.width = shaderDiv.clientWidth;
      gl.canvas.height = shaderDiv.clientHeight;
      gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    }

    gl.uniform2f(shaderCanvasSizeUniformLocation, gl.canvas.width, gl.canvas.height);
    gl.uniform2f(shaderScreenSizeUniformLocation, screen.width * window.devicePixelRatio, screen.height * window.devicePixelRatio);
    gl.uniform1f(shaderScaleUniformLocation, shaderScale);

    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  // request the next frame
  requestAnimationFrame(render);
}