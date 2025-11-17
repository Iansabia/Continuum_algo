import{r as x,j as f,a as Et,m as I,c as q,p as ie,b as Ot,d as Tt,R as Rt}from"./index-98Kq2bIe.js";const kt=`#version 300 es
precision mediump float;

layout(location = 0) in vec4 a_position;

uniform vec2 u_resolution;
uniform float u_pixelRatio;
uniform float u_imageAspectRatio;

uniform float u_originX;
uniform float u_originY;
uniform float u_worldWidth;
uniform float u_worldHeight;
uniform float u_fit;

uniform float u_scale;
uniform float u_rotation;
uniform float u_offsetX;
uniform float u_offsetY;

uniform float u_pxSize;

out vec2 v_objectUV;
out vec2 v_objectBoxSize;
out vec2 v_objectHelperBox;

out vec2 v_responsiveUV;
out vec2 v_responsiveBoxSize;
out vec2 v_responsiveHelperBox;
out vec2 v_responsiveBoxGivenSize;

out vec2 v_patternUV;
out vec2 v_patternBoxSize;
out vec2 v_patternHelperBox;

out vec2 v_imageUV;

// #define ADD_HELPERS

vec3 getBoxSize(float boxRatio, vec2 givenBoxSize) {
  vec2 box = vec2(0.);
  // fit = none
  box.x = boxRatio * min(givenBoxSize.x / boxRatio, givenBoxSize.y);
  float noFitBoxWidth = box.x;
  if (u_fit == 1.) { // fit = contain
    box.x = boxRatio * min(u_resolution.x / boxRatio, u_resolution.y);
  } else if (u_fit == 2.) { // fit = cover
    box.x = boxRatio * max(u_resolution.x / boxRatio, u_resolution.y);
  }
  box.y = box.x / boxRatio;
  return vec3(box, noFitBoxWidth);
}

void main() {
  gl_Position = a_position;

  vec2 uv = gl_Position.xy * .5;
  vec2 boxOrigin = vec2(.5 - u_originX, u_originY - .5);
  vec2 givenBoxSize = vec2(u_worldWidth, u_worldHeight);
  givenBoxSize = max(givenBoxSize, vec2(1.)) * u_pixelRatio;
  float r = u_rotation * 3.14159265358979323846 / 180.;
  mat2 graphicRotation = mat2(cos(r), sin(r), -sin(r), cos(r));
  vec2 graphicOffset = vec2(-u_offsetX, u_offsetY);


  // ===================================================
  // Sizing api for graphic objects with fixed ratio
  // (currently supports only ratio = 1)

  float fixedRatio = 1.;
  vec2 fixedRatioBoxGivenSize = vec2(
  (u_worldWidth == 0.) ? u_resolution.x : givenBoxSize.x,
  (u_worldHeight == 0.) ? u_resolution.y : givenBoxSize.y
  );

  v_objectBoxSize = getBoxSize(fixedRatio, fixedRatioBoxGivenSize).xy;
  vec2 objectWorldScale = u_resolution.xy / v_objectBoxSize;

  #ifdef ADD_HELPERS
  v_objectHelperBox = uv;
  v_objectHelperBox *= objectWorldScale;
  v_objectHelperBox += boxOrigin * (objectWorldScale - 1.);
  #endif

  v_objectUV = uv;
  v_objectUV *= objectWorldScale;
  v_objectUV += boxOrigin * (objectWorldScale - 1.);
  v_objectUV += graphicOffset;
  v_objectUV /= u_scale;
  v_objectUV = graphicRotation * v_objectUV;


  // ===================================================


  // ===================================================
  // Sizing api for graphic objects with either givenBoxSize ratio or canvas ratio.
  // Full-screen mode available with u_worldWidth = u_worldHeight = 0

  v_responsiveBoxGivenSize = vec2(
  (u_worldWidth == 0.) ? u_resolution.x : givenBoxSize.x,
  (u_worldHeight == 0.) ? u_resolution.y : givenBoxSize.y
  );
  float responsiveRatio = v_responsiveBoxGivenSize.x / v_responsiveBoxGivenSize.y;
  v_responsiveBoxSize = getBoxSize(responsiveRatio, v_responsiveBoxGivenSize).xy;
  vec2 responsiveBoxScale = u_resolution.xy / v_responsiveBoxSize;

  #ifdef ADD_HELPERS
  v_responsiveHelperBox = uv;
  v_responsiveHelperBox *= responsiveBoxScale;
  v_responsiveHelperBox += boxOrigin * (responsiveBoxScale - 1.);
  #endif

  v_responsiveUV = uv;
  v_responsiveUV *= responsiveBoxScale;
  v_responsiveUV += boxOrigin * (responsiveBoxScale - 1.);
  v_responsiveUV += graphicOffset;
  v_responsiveUV /= u_scale;
  v_responsiveUV.x *= responsiveRatio;
  v_responsiveUV = graphicRotation * v_responsiveUV;
  v_responsiveUV.x /= responsiveRatio;

  // ===================================================


  // ===================================================
  // Sizing api for patterns
  // (treating graphics as a image u_worldWidth x u_worldHeight size)

  float patternBoxRatio = givenBoxSize.x / givenBoxSize.y;
  vec2 patternBoxGivenSize = vec2(
  (u_worldWidth == 0.) ? u_resolution.x : givenBoxSize.x,
  (u_worldHeight == 0.) ? u_resolution.y : givenBoxSize.y
  );
  patternBoxRatio = patternBoxGivenSize.x / patternBoxGivenSize.y;

  vec3 boxSizeData = getBoxSize(patternBoxRatio, patternBoxGivenSize);
  v_patternBoxSize = boxSizeData.xy;
  float patternBoxNoFitBoxWidth = boxSizeData.z;
  vec2 patternBoxScale = u_resolution.xy / v_patternBoxSize;

  #ifdef ADD_HELPERS
  v_patternHelperBox = uv;
  v_patternHelperBox *= patternBoxScale;
  v_patternHelperBox += boxOrigin * (patternBoxScale - 1.);
  #endif

  v_patternUV = uv;
  v_patternUV += graphicOffset / patternBoxScale;
  v_patternUV += boxOrigin;
  v_patternUV -= boxOrigin / patternBoxScale;
  v_patternUV *= u_resolution.xy;
  v_patternUV /= u_pixelRatio;
  if (u_fit > 0.) {
    v_patternUV *= (patternBoxNoFitBoxWidth / v_patternBoxSize.x);
  }
  v_patternUV /= u_scale;
  v_patternUV = graphicRotation * v_patternUV;
  v_patternUV += boxOrigin / patternBoxScale;
  v_patternUV -= boxOrigin;
  // x100 is a default multiplier between vertex and fragmant shaders
  // we use it to avoid UV presision issues
  v_patternUV *= .01;

  // ===================================================


  // ===================================================
  // Sizing api for images

  vec2 imageBoxSize;
  if (u_fit == 1.) { // contain
    imageBoxSize.x = min(u_resolution.x / u_imageAspectRatio, u_resolution.y) * u_imageAspectRatio;
  } else if (u_fit == 2.) { // cover
    imageBoxSize.x = max(u_resolution.x / u_imageAspectRatio, u_resolution.y) * u_imageAspectRatio;
  } else {
    imageBoxSize.x = min(10.0, 10.0 / u_imageAspectRatio * u_imageAspectRatio);
  }
  imageBoxSize.y = imageBoxSize.x / u_imageAspectRatio;
  vec2 imageBoxScale = u_resolution.xy / imageBoxSize;

  #ifdef ADD_HELPERS
  vec2 imageHelperBox = uv;
  imageHelperBox *= imageBoxScale;
  imageHelperBox += boxOrigin * (imageBoxScale - 1.);
  #endif

  v_imageUV = uv;
  v_imageUV *= imageBoxScale;
  v_imageUV += boxOrigin * (imageBoxScale - 1.);
  v_imageUV += graphicOffset;
  v_imageUV /= u_scale;
  v_imageUV.x *= u_imageAspectRatio;
  v_imageUV = graphicRotation * v_imageUV;
  v_imageUV.x /= u_imageAspectRatio;

  v_imageUV += .5;
  v_imageUV.y = 1. - v_imageUV.y;

  // ===================================================

}`,Ue=1920*1080*4;let Pt=class{parentElement;canvasElement;gl;program=null;uniformLocations={};fragmentShader;rafId=null;lastRenderTime=0;currentFrame=0;speed=0;currentSpeed=0;providedUniforms;mipmaps=[];hasBeenDisposed=!1;resolutionChanged=!0;textures=new Map;minPixelRatio;maxPixelCount;isSafari=Bt();uniformCache={};textureUnitMap=new Map;constructor(e,r,n,i,s=0,a=0,o=2,l=Ue,c=[]){if(e instanceof HTMLElement)this.parentElement=e;else throw new Error("Paper Shaders: parent element must be an HTMLElement");if(!document.querySelector("style[data-paper-shader]")){const h=document.createElement("style");h.innerHTML=Ut,h.setAttribute("data-paper-shader",""),document.head.prepend(h)}const u=document.createElement("canvas");this.canvasElement=u,this.parentElement.prepend(u),this.fragmentShader=r,this.providedUniforms=n,this.mipmaps=c,this.currentFrame=a,this.minPixelRatio=o,this.maxPixelCount=l;const d=u.getContext("webgl2",i);if(!d)throw new Error("Paper Shaders: WebGL is not supported in this browser");this.gl=d,this.initProgram(),this.setupPositionAttribute(),this.setupUniforms(),this.setUniformValues(this.providedUniforms),this.setupResizeObserver(),visualViewport?.addEventListener("resize",this.handleVisualViewportChange),this.setSpeed(s),this.parentElement.setAttribute("data-paper-shader",""),this.parentElement.paperShaderMount=this,document.addEventListener("visibilitychange",this.handleDocumentVisibilityChange)}initProgram=()=>{const e=jt(this.gl,kt,this.fragmentShader);e&&(this.program=e)};setupPositionAttribute=()=>{const e=this.gl.getAttribLocation(this.program,"a_position"),r=this.gl.createBuffer();this.gl.bindBuffer(this.gl.ARRAY_BUFFER,r);const n=[-1,-1,1,-1,-1,1,-1,1,1,-1,1,1];this.gl.bufferData(this.gl.ARRAY_BUFFER,new Float32Array(n),this.gl.STATIC_DRAW),this.gl.enableVertexAttribArray(e),this.gl.vertexAttribPointer(e,2,this.gl.FLOAT,!1,0,0)};setupUniforms=()=>{const e={u_time:this.gl.getUniformLocation(this.program,"u_time"),u_pixelRatio:this.gl.getUniformLocation(this.program,"u_pixelRatio"),u_resolution:this.gl.getUniformLocation(this.program,"u_resolution")};Object.entries(this.providedUniforms).forEach(([r,n])=>{if(e[r]=this.gl.getUniformLocation(this.program,r),n instanceof HTMLImageElement){const i=`${r}AspectRatio`;e[i]=this.gl.getUniformLocation(this.program,i)}}),this.uniformLocations=e};renderScale=1;parentWidth=0;parentHeight=0;parentDevicePixelWidth=0;parentDevicePixelHeight=0;devicePixelsSupported=!1;resizeObserver=null;setupResizeObserver=()=>{this.resizeObserver=new ResizeObserver(([e])=>{if(e?.borderBoxSize[0]){const r=e.devicePixelContentBoxSize?.[0];r!==void 0&&(this.devicePixelsSupported=!0,this.parentDevicePixelWidth=r.inlineSize,this.parentDevicePixelHeight=r.blockSize),this.parentWidth=e.borderBoxSize[0].inlineSize,this.parentHeight=e.borderBoxSize[0].blockSize}this.handleResize()}),this.resizeObserver.observe(this.parentElement)};handleVisualViewportChange=()=>{this.resizeObserver?.disconnect(),this.setupResizeObserver()};handleResize=()=>{let e=0,r=0;const n=Math.max(1,window.devicePixelRatio),i=visualViewport?.scale??1;if(this.devicePixelsSupported){const u=Math.max(1,this.minPixelRatio/n);e=this.parentDevicePixelWidth*u*i,r=this.parentDevicePixelHeight*u*i}else{let u=Math.max(n,this.minPixelRatio)*i;if(this.isSafari){const d=Mt();u*=Math.max(1,d)}e=Math.round(this.parentWidth)*u,r=Math.round(this.parentHeight)*u}const s=Math.sqrt(this.maxPixelCount)/Math.sqrt(e*r),a=Math.min(1,s),o=Math.round(e*a),l=Math.round(r*a),c=o/Math.round(this.parentWidth);(this.canvasElement.width!==o||this.canvasElement.height!==l||this.renderScale!==c)&&(this.renderScale=c,this.canvasElement.width=o,this.canvasElement.height=l,this.resolutionChanged=!0,this.gl.viewport(0,0,this.gl.canvas.width,this.gl.canvas.height),this.render(performance.now()))};render=e=>{if(this.hasBeenDisposed)return;if(this.program===null){console.warn("Tried to render before program or gl was initialized");return}const r=e-this.lastRenderTime;this.lastRenderTime=e,this.currentSpeed!==0&&(this.currentFrame+=r*this.currentSpeed),this.gl.clear(this.gl.COLOR_BUFFER_BIT),this.gl.useProgram(this.program),this.gl.uniform1f(this.uniformLocations.u_time,this.currentFrame*.001),this.resolutionChanged&&(this.gl.uniform2f(this.uniformLocations.u_resolution,this.gl.canvas.width,this.gl.canvas.height),this.gl.uniform1f(this.uniformLocations.u_pixelRatio,this.renderScale),this.resolutionChanged=!1),this.gl.drawArrays(this.gl.TRIANGLES,0,6),this.currentSpeed!==0?this.requestRender():this.rafId=null};requestRender=()=>{this.rafId!==null&&cancelAnimationFrame(this.rafId),this.rafId=requestAnimationFrame(this.render)};setTextureUniform=(e,r)=>{if(!r.complete||r.naturalWidth===0)throw new Error(`Paper Shaders: image for uniform ${e} must be fully loaded`);const n=this.textures.get(e);n&&this.gl.deleteTexture(n),this.textureUnitMap.has(e)||this.textureUnitMap.set(e,this.textureUnitMap.size);const i=this.textureUnitMap.get(e);this.gl.activeTexture(this.gl.TEXTURE0+i);const s=this.gl.createTexture();this.gl.bindTexture(this.gl.TEXTURE_2D,s),this.gl.texParameteri(this.gl.TEXTURE_2D,this.gl.TEXTURE_WRAP_S,this.gl.CLAMP_TO_EDGE),this.gl.texParameteri(this.gl.TEXTURE_2D,this.gl.TEXTURE_WRAP_T,this.gl.CLAMP_TO_EDGE),this.gl.texParameteri(this.gl.TEXTURE_2D,this.gl.TEXTURE_MIN_FILTER,this.gl.LINEAR),this.gl.texParameteri(this.gl.TEXTURE_2D,this.gl.TEXTURE_MAG_FILTER,this.gl.LINEAR),this.gl.texImage2D(this.gl.TEXTURE_2D,0,this.gl.RGBA,this.gl.RGBA,this.gl.UNSIGNED_BYTE,r),this.mipmaps.includes(e)&&(this.gl.generateMipmap(this.gl.TEXTURE_2D),this.gl.texParameteri(this.gl.TEXTURE_2D,this.gl.TEXTURE_MIN_FILTER,this.gl.LINEAR_MIPMAP_LINEAR));const a=this.gl.getError();if(a!==this.gl.NO_ERROR||s===null){console.error("Paper Shaders: WebGL error when uploading texture:",a);return}this.textures.set(e,s);const o=this.uniformLocations[e];if(o){this.gl.uniform1i(o,i);const l=`${e}AspectRatio`,c=this.uniformLocations[l];if(c){const u=r.naturalWidth/r.naturalHeight;this.gl.uniform1f(c,u)}}};areUniformValuesEqual=(e,r)=>e===r?!0:Array.isArray(e)&&Array.isArray(r)&&e.length===r.length?e.every((n,i)=>this.areUniformValuesEqual(n,r[i])):!1;setUniformValues=e=>{this.gl.useProgram(this.program),Object.entries(e).forEach(([r,n])=>{let i=n;if(n instanceof HTMLImageElement&&(i=`${n.src.slice(0,200)}|${n.naturalWidth}x${n.naturalHeight}`),this.areUniformValuesEqual(this.uniformCache[r],i))return;this.uniformCache[r]=i;const s=this.uniformLocations[r];if(!s){console.warn(`Uniform location for ${r} not found`);return}if(n instanceof HTMLImageElement)this.setTextureUniform(r,n);else if(Array.isArray(n)){let a=null,o=null;if(n[0]!==void 0&&Array.isArray(n[0])){const l=n[0].length;if(n.every(c=>c.length===l))a=n.flat(),o=l;else{console.warn(`All child arrays must be the same length for ${r}`);return}}else a=n,o=a.length;switch(o){case 2:this.gl.uniform2fv(s,a);break;case 3:this.gl.uniform3fv(s,a);break;case 4:this.gl.uniform4fv(s,a);break;case 9:this.gl.uniformMatrix3fv(s,!1,a);break;case 16:this.gl.uniformMatrix4fv(s,!1,a);break;default:console.warn(`Unsupported uniform array length: ${o}`)}}else typeof n=="number"?this.gl.uniform1f(s,n):typeof n=="boolean"?this.gl.uniform1i(s,n?1:0):console.warn(`Unsupported uniform type for ${r}: ${typeof n}`)})};getCurrentFrame=()=>this.currentFrame;setFrame=e=>{this.currentFrame=e,this.lastRenderTime=performance.now(),this.render(performance.now())};setSpeed=(e=1)=>{this.speed=e,this.setCurrentSpeed(document.hidden?0:e)};setCurrentSpeed=e=>{this.currentSpeed=e,this.rafId===null&&e!==0&&(this.lastRenderTime=performance.now(),this.rafId=requestAnimationFrame(this.render)),this.rafId!==null&&e===0&&(cancelAnimationFrame(this.rafId),this.rafId=null)};setMaxPixelCount=(e=Ue)=>{this.maxPixelCount=e,this.handleResize()};setMinPixelRatio=(e=2)=>{this.minPixelRatio=e,this.handleResize()};setUniforms=e=>{this.setUniformValues(e),this.providedUniforms={...this.providedUniforms,...e},this.render(performance.now())};handleDocumentVisibilityChange=()=>{this.setCurrentSpeed(document.hidden?0:this.speed)};dispose=()=>{this.hasBeenDisposed=!0,this.rafId!==null&&(cancelAnimationFrame(this.rafId),this.rafId=null),this.gl&&this.program&&(this.textures.forEach(e=>{this.gl.deleteTexture(e)}),this.textures.clear(),this.gl.deleteProgram(this.program),this.program=null,this.gl.bindBuffer(this.gl.ARRAY_BUFFER,null),this.gl.bindBuffer(this.gl.ELEMENT_ARRAY_BUFFER,null),this.gl.bindRenderbuffer(this.gl.RENDERBUFFER,null),this.gl.bindFramebuffer(this.gl.FRAMEBUFFER,null),this.gl.getError()),this.resizeObserver&&(this.resizeObserver.disconnect(),this.resizeObserver=null),visualViewport?.removeEventListener("resize",this.handleVisualViewportChange),document.removeEventListener("visibilitychange",this.handleDocumentVisibilityChange),this.uniformLocations={},this.canvasElement.remove(),delete this.parentElement.paperShaderMount}};function Be(t,e,r){const n=t.createShader(e);return n?(t.shaderSource(n,r),t.compileShader(n),t.getShaderParameter(n,t.COMPILE_STATUS)?n:(console.error("An error occurred compiling the shaders: "+t.getShaderInfoLog(n)),t.deleteShader(n),null)):null}function jt(t,e,r){const n=t.getShaderPrecisionFormat(t.FRAGMENT_SHADER,t.MEDIUM_FLOAT),i=n?n.precision:null;i&&i<23&&(e=e.replace(/precision\s+(lowp|mediump)\s+float;/g,"precision highp float;"),r=r.replace(/precision\s+(lowp|mediump)\s+float/g,"precision highp float").replace(/\b(uniform|varying|attribute)\s+(lowp|mediump)\s+(\w+)/g,"$1 highp $3"));const s=Be(t,t.VERTEX_SHADER,e),a=Be(t,t.FRAGMENT_SHADER,r);if(!s||!a)return null;const o=t.createProgram();return o?(t.attachShader(o,s),t.attachShader(o,a),t.linkProgram(o),t.getProgramParameter(o,t.LINK_STATUS)?(t.detachShader(o,s),t.detachShader(o,a),t.deleteShader(s),t.deleteShader(a),o):(console.error("Unable to initialize the shader program: "+t.getProgramInfoLog(o)),t.deleteProgram(o),t.deleteShader(s),t.deleteShader(a),null)):null}const Ut=`@layer paper-shaders {
  :where([data-paper-shader]) {
    isolation: isolate;
    position: relative;

    & canvas {
      contain: strict;
      display: block;
      position: absolute;
      inset: 0;
      z-index: -1;
      width: 100%;
      height: 100%;
      border-radius: inherit;
      corner-shape: inherit;
    }
  }
}`;function Bt(){const t=navigator.userAgent.toLowerCase();return t.includes("safari")&&!t.includes("chrome")&&!t.includes("android")}function Mt(){const t=visualViewport?.scale??1,e=visualViewport?.width??window.innerWidth,r=window.innerWidth-document.documentElement.clientWidth,n=t*e+r,i=outerWidth/n,s=Math.round(100*i);return s%5===0?s/100:s===33?1/3:s===67?2/3:s===133?4/3:i}const At=`
in vec2 v_objectUV;
in vec2 v_responsiveUV;
in vec2 v_responsiveBoxGivenSize;
in vec2 v_patternUV;
in vec2 v_imageUV;`,It=`
in vec2 v_objectBoxSize;
in vec2 v_objectHelperBox;
in vec2 v_responsiveBoxSize;
in vec2 v_responsiveHelperBox;
in vec2 v_patternBoxSize;
in vec2 v_patternHelperBox;`,Ht=`
uniform float u_originX;
uniform float u_originY;
uniform float u_worldWidth;
uniform float u_worldHeight;
uniform float u_fit;

uniform float u_scale;
uniform float u_rotation;
uniform float u_offsetX;
uniform float u_offsetY;`,Lt={fit:"contain",scale:1,rotation:0,offsetX:0,offsetY:0,originX:.5,originY:.5,worldWidth:0,worldHeight:0},zt={none:0,contain:1,cover:2},Ct=`
#define TWO_PI 6.28318530718
#define PI 3.14159265358979323846
`,$t=`
vec2 rotate(vec2 uv, float th) {
  return mat2(cos(th), sin(th), -sin(th), cos(th)) * uv;
}
`,Vt=`
  float hash21(vec2 p) {
    p = fract(p * vec2(0.3183099, 0.3678794)) + 0.1;
    p += dot(p, p + 19.19);
    return fract(p.x * p.y);
  }
`,Me={maxColorCount:10},Dt=`#version 300 es
precision mediump float;

uniform float u_time;

uniform vec4 u_colors[${Me.maxColorCount}];
uniform float u_colorsCount;

uniform float u_distortion;
uniform float u_swirl;
uniform float u_grainMixer;
uniform float u_grainOverlay;

${At}
${It}
${Ht}

out vec4 fragColor;

${Ct}
${$t}
${Vt}

float valueNoise(vec2 st) {
  vec2 i = floor(st);
  vec2 f = fract(st);
  float a = hash21(i);
  float b = hash21(i + vec2(1.0, 0.0));
  float c = hash21(i + vec2(0.0, 1.0));
  float d = hash21(i + vec2(1.0, 1.0));
  vec2 u = f * f * (3.0 - 2.0 * f);
  float x1 = mix(a, b, u.x);
  float x2 = mix(c, d, u.x);
  return mix(x1, x2, u.y);
}

float noise(vec2 n, vec2 seedOffset) {
  return valueNoise(n + seedOffset);
}

vec2 getPosition(int i, float t) {
  float a = float(i) * .37;
  float b = .6 + fract(float(i) / 3.) * .9;
  float c = .8 + fract(float(i + 1) / 4.);

  float x = sin(t * b + a);
  float y = cos(t * c + a * 1.5);

  return .5 + .5 * vec2(x, y);
}

void main() {
  vec2 shape_uv = v_objectUV;
  shape_uv += .5;

  vec2 grainUV = v_objectUV;
  // apply inverse transform to grain_uv so it respects the originXY
  float grainUVRot = u_rotation * 3.14159265358979323846 / 180.;
  mat2 graphicRotation = mat2(cos(grainUVRot), sin(grainUVRot), -sin(grainUVRot), cos(grainUVRot));
  vec2 graphicOffset = vec2(-u_offsetX, u_offsetY);
  grainUV = transpose(graphicRotation) * grainUV;
  grainUV *= u_scale;
  grainUV *= .7;
  grainUV -= graphicOffset;
  grainUV *= v_objectBoxSize;
  
  float grain = noise(grainUV, vec2(0.));
  float mixerGrain = .4 * u_grainMixer * (grain - .5);

  const float firstFrameOffset = 41.5;
  float t = .5 * (u_time + firstFrameOffset);

  float radius = smoothstep(0., 1., length(shape_uv - .5));
  float center = 1. - radius;
  for (float i = 1.; i <= 2.; i++) {
    shape_uv.x += u_distortion * center / i * sin(t + i * .4 * smoothstep(.0, 1., shape_uv.y)) * cos(.2 * t + i * 2.4 * smoothstep(.0, 1., shape_uv.y));
    shape_uv.y += u_distortion * center / i * cos(t + i * 2. * smoothstep(.0, 1., shape_uv.x));
  }

  vec2 uvRotated = shape_uv;
  uvRotated -= vec2(.5);
  float angle = 3. * u_swirl * radius;
  uvRotated = rotate(uvRotated, -angle);
  uvRotated += vec2(.5);

  vec3 color = vec3(0.);
  float opacity = 0.;
  float totalWeight = 0.;

  for (int i = 0; i < ${Me.maxColorCount}; i++) {
    if (i >= int(u_colorsCount)) break;

    vec2 pos = getPosition(i, t) + mixerGrain;
    vec3 colorFraction = u_colors[i].rgb * u_colors[i].a;
    float opacityFraction = u_colors[i].a;

    float dist = length(uvRotated - pos);

    dist = pow(dist, 3.5);
    float weight = 1. / (dist + 1e-3);
    color += colorFraction * weight;
    opacity += opacityFraction * weight;
    totalWeight += weight;
  }

  color /= max(1e-4, totalWeight);
  opacity /= max(1e-4, totalWeight);

  float rr = noise(rotate(grainUV, 1.), vec2(3.));
  float gg = noise(rotate(grainUV, 2.) + 10., vec2(-1.));
  float bb = noise(grainUV - 2., vec2(5.));
  vec3 grainColor = vec3(rr, gg, bb);
  color = mix(color, grainColor, .01 + .3 * u_grainOverlay);
  
  fragColor = vec4(color, opacity);
}
`;function Nt(t){if(Array.isArray(t))return t.length===4?t:t.length===3?[...t,1]:fe;if(typeof t!="string")return fe;let e,r,n,i=1;if(t.startsWith("#"))[e,r,n,i]=Wt(t);else if(t.startsWith("rgb"))[e,r,n,i]=Ft(t);else if(t.startsWith("hsl"))[e,r,n,i]=Gt(Xt(t));else return console.error("Unsupported color format",t),fe;return[Q(e,0,1),Q(r,0,1),Q(n,0,1),Q(i,0,1)]}function Wt(t){t=t.replace(/^#/,""),t.length===3&&(t=t.split("").map(s=>s+s).join("")),t.length===6&&(t=t+"ff");const e=parseInt(t.slice(0,2),16)/255,r=parseInt(t.slice(2,4),16)/255,n=parseInt(t.slice(4,6),16)/255,i=parseInt(t.slice(6,8),16)/255;return[e,r,n,i]}function Ft(t){const e=t.match(/^rgba?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([0-9.]+))?\s*\)$/i);return e?[parseInt(e[1]??"0")/255,parseInt(e[2]??"0")/255,parseInt(e[3]??"0")/255,e[4]===void 0?1:parseFloat(e[4])]:[0,0,0,1]}function Xt(t){const e=t.match(/^hsla?\s*\(\s*(\d+)\s*,\s*(\d+)%\s*,\s*(\d+)%\s*(?:,\s*([0-9.]+))?\s*\)$/i);return e?[parseInt(e[1]??"0"),parseInt(e[2]??"0"),parseInt(e[3]??"0"),e[4]===void 0?1:parseFloat(e[4])]:[0,0,0,1]}function Gt(t){const[e,r,n,i]=t,s=e/360,a=r/100,o=n/100;let l,c,u;if(r===0)l=c=u=o;else{const d=(y,E,p)=>(p<0&&(p+=1),p>1&&(p-=1),p<.16666666666666666?y+(E-y)*6*p:p<.5?E:p<.6666666666666666?y+(E-y)*(.6666666666666666-p)*6:y),h=o<.5?o*(1+a):o+a-o*a,S=2*o-h;l=d(S,h,s+1/3),c=d(S,h,s),u=d(S,h,s-1/3)}return[l,c,u,i]}const Q=(t,e,r)=>Math.min(Math.max(t,e),r),fe=[0,0,0,1];function Yt(){if(typeof window>"u"){console.warn("Paper Shaders: can’t create an image on the server");return}const t=new Image;return t.src=qt,t}const qt="data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw==";function Qt(t){const e=x.useRef(void 0),r=x.useCallback(n=>{const i=t.map(s=>{if(s!=null){if(typeof s=="function"){const a=s,o=a(n);return typeof o=="function"?o:()=>{a(null)}}return s.current=n,()=>{s.current=null}}});return()=>{i.forEach(s=>s?.())}},t);return x.useMemo(()=>t.every(n=>n==null)?null:n=>{e.current&&(e.current(),e.current=void 0),n!=null&&(e.current=r(n))},t)}async function Ae(t){const e={},r=[],n=s=>{try{return s.startsWith("/")||new URL(s),!0}catch{return!1}},i=s=>{try{return s.startsWith("/")?!1:new URL(s,window.location.origin).origin!==window.location.origin}catch{return!1}};return Object.entries(t).forEach(([s,a])=>{if(typeof a=="string"){if(!a){e[s]=Yt();return}if(!n(a)){console.warn(`Uniform "${s}" has invalid URL "${a}". Skipping image loading.`);return}const o=new Promise((l,c)=>{const u=new Image;i(a)&&(u.crossOrigin="anonymous"),u.onload=()=>{e[s]=u,l()},u.onerror=()=>{console.error(`Could not set uniforms. Failed to load image at ${a}`),c()},u.src=a});r.push(o)}else e[s]=a}),await Promise.all(r),e}const rt=x.forwardRef(function({fragmentShader:e,uniforms:r,webGlContextAttributes:n,speed:i=0,frame:s=0,width:a,height:o,minPixelRatio:l,maxPixelCount:c,mipmaps:u,style:d,...h},S){const[y,E]=x.useState(!1),p=x.useRef(null),v=x.useRef(null),P=x.useRef(n);x.useEffect(()=>((async()=>{const R=await Ae(r);p.current&&!v.current&&(v.current=new Pt(p.current,e,R,P.current,i,s,l,c,u),E(!0))})(),()=>{v.current?.dispose(),v.current=null}),[e]),x.useEffect(()=>{let T=!1;return(async()=>{const m=await Ae(r);T||v.current?.setUniforms(m)})(),()=>{T=!0}},[r,y]),x.useEffect(()=>{v.current?.setSpeed(i)},[i,y]),x.useEffect(()=>{v.current?.setMaxPixelCount(c)},[c,y]),x.useEffect(()=>{v.current?.setMinPixelRatio(l)},[l,y]),x.useEffect(()=>{v.current?.setFrame(s)},[s,y]);const B=Qt([p,S]);return f.jsx("div",{ref:B,style:a!==void 0||o!==void 0?{width:typeof a=="string"&&isNaN(+a)===!1?+a:a,height:typeof o=="string"&&isNaN(+o)===!1?+o:o,...d}:d,...h})});rt.displayName="ShaderMount";function Kt(t,e){for(const r in t){if(r==="colors"){const n=Array.isArray(t.colors),i=Array.isArray(e.colors);if(!n||!i){if(Object.is(t.colors,e.colors)===!1)return!1;continue}if(t.colors?.length!==e.colors?.length||!t.colors?.every((s,a)=>s===e.colors?.[a]))return!1;continue}if(Object.is(t[r],e[r])===!1)return!1}return!0}const O={params:{...Lt,speed:1,frame:0,colors:["#e0eaff","#241d9a","#f75092","#9f50d3"],distortion:.8,swirl:.1,grainMixer:0,grainOverlay:0}},Zt=x.memo(function({speed:e=O.params.speed,frame:r=O.params.frame,colors:n=O.params.colors,distortion:i=O.params.distortion,swirl:s=O.params.swirl,grainMixer:a=O.params.grainMixer,grainOverlay:o=O.params.grainOverlay,fit:l=O.params.fit,rotation:c=O.params.rotation,scale:u=O.params.scale,originX:d=O.params.originX,originY:h=O.params.originY,offsetX:S=O.params.offsetX,offsetY:y=O.params.offsetY,worldWidth:E=O.params.worldWidth,worldHeight:p=O.params.worldHeight,...v}){const P={u_colors:n.map(Nt),u_colorsCount:n.length,u_distortion:i,u_swirl:s,u_grainMixer:a,u_grainOverlay:o,u_fit:zt[l],u_rotation:c,u_scale:u,u_offsetX:S,u_offsetY:y,u_originX:d,u_originY:h,u_worldWidth:E,u_worldHeight:p};return f.jsx(rt,{...v,speed:e,frame:r,fragmentShader:Dt,uniforms:P})},Kt),Jt={some:0,all:1};function er(t,e,{root:r,margin:n,amount:i="some"}={}){const s=Et(t),a=new WeakMap,o=c=>{c.forEach(u=>{const d=a.get(u.target);if(u.isIntersecting!==!!d)if(u.isIntersecting){const h=e(u.target,u);typeof h=="function"?a.set(u.target,h):l.unobserve(u.target)}else typeof d=="function"&&(d(u),a.delete(u.target))})},l=new IntersectionObserver(o,{root:r,rootMargin:n,threshold:typeof i=="number"?i:Jt[i]});return s.forEach(c=>l.observe(c)),()=>l.disconnect()}function tr(t,{root:e,margin:r,amount:n,once:i=!1,initial:s=!1}={}){const[a,o]=x.useState(s);return x.useEffect(()=>{if(!t.current||i&&a)return;const l=()=>(o(!0),i?void 0:()=>o(!1)),c={root:e&&e.current||void 0,margin:r,amount:n};return er(t.current,l,c)},[e,t,r,i,n]),a}const de=({children:t,onClick:e,href:r,variant:n="primary",className:i=""})=>{const s=n==="primary",a={position:"relative",padding:"16px 40px",fontSize:"15.5px",fontWeight:680,letterSpacing:"0.5px",border:"none",cursor:"pointer",textDecoration:"none",display:"inline-block",borderRadius:"12px",fontFamily:'Inter, -apple-system, BlinkMacSystemFont, "SF Pro Display", sans-serif',WebkitFontSmoothing:"antialiased",MozOsxFontSmoothing:"grayscale",transition:"all 0.2s ease"},o={background:`
      linear-gradient(135deg,
        #fcfcfd 0%,
        #f8f8fa 15%,
        #f3f4f6 30%,
        #eeeff2 45%,
        #e9eaed 60%,
        #e4e5e8 75%,
        #dee0e3 90%,
        #e2e3e6 100%
      )
    `,color:"#1a1a1a",boxShadow:`
      0 3px 6px rgba(0, 0, 0, 0.12),
      0 8px 16px rgba(0, 0, 0, 0.10),
      0 16px 32px rgba(0, 0, 0, 0.08),
      0 1px 2px rgba(0, 0, 0, 0.12),
      inset 0 2px 1px rgba(255, 255, 255, 0.7),
      inset 0 -2px 6px rgba(0, 0, 0, 0.10),
      inset 2px 2px 8px rgba(0, 0, 0, 0.08),
      inset -2px 2px 8px rgba(0, 0, 0, 0.07),
      inset 0 0 1px rgba(0, 0, 0, 0.15)
    `,textShadow:`
      0 1px 0 rgba(0, 0, 0, 0.35),
      0 -1px 0 rgba(255, 255, 255, 0.8),
      1px 1px 0 rgba(0, 0, 0, 0.18),
      -1px 1px 0 rgba(0, 0, 0, 0.15)
    `},l={background:`
      linear-gradient(135deg,
        rgba(252, 252, 253, 0.4) 0%,
        rgba(248, 248, 250, 0.4) 50%,
        rgba(244, 244, 246, 0.4) 100%
      )
    `,color:"#1a1a1a",border:"2px solid rgba(255, 255, 255, 0.3)",backdropFilter:"blur(10px)",boxShadow:`
      0 2px 4px rgba(0, 0, 0, 0.08),
      0 4px 8px rgba(0, 0, 0, 0.06),
      inset 0 1px 1px rgba(255, 255, 255, 0.6),
      inset 0 -1px 3px rgba(0, 0, 0, 0.06)
    `,textShadow:`
      0 1px 0 rgba(0, 0, 0, 0.25),
      0 -1px 0 rgba(255, 255, 255, 0.7)
    `},c=s?{transform:"translateY(-2px)",boxShadow:`
          0 4px 8px rgba(0, 0, 0, 0.14),
          0 10px 20px rgba(0, 0, 0, 0.12),
          0 20px 40px rgba(0, 0, 0, 0.10),
          inset 0 2px 2px rgba(255, 255, 255, 0.8),
          inset 0 -3px 8px rgba(0, 0, 0, 0.12)
        `}:{transform:"translateY(-2px)",background:`
          linear-gradient(135deg,
            rgba(252, 252, 253, 0.6) 0%,
            rgba(248, 248, 250, 0.6) 50%,
            rgba(244, 244, 246, 0.6) 100%
          )
        `,borderColor:"rgba(255, 255, 255, 0.5)"},u={transform:"translateY(1px)",boxShadow:s?`
        0 1px 2px rgba(0, 0, 0, 0.10),
        0 2px 4px rgba(0, 0, 0, 0.08),
        inset 0 2px 4px rgba(0, 0, 0, 0.12)
      `:`
        0 1px 2px rgba(0, 0, 0, 0.06),
        inset 0 1px 2px rgba(0, 0, 0, 0.08)
      `},d={...a,...s?o:l},h=f.jsxs(f.Fragment,{children:[f.jsx("div",{className:"absolute inset-x-0 top-0 rounded-t-xl pointer-events-none",style:{height:"2px",background:"linear-gradient(90deg, rgba(255, 255, 255, 0) 0%, rgba(255, 255, 255, 0.9) 20%, rgba(255, 255, 255, 1) 50%, rgba(255, 255, 255, 0.9) 80%, rgba(255, 255, 255, 0) 100%)",filter:"blur(0.3px)"}}),f.jsx("div",{className:"absolute inset-x-0 top-0 rounded-xl pointer-events-none",style:{height:"50%",background:"linear-gradient(180deg, rgba(255, 255, 255, 0.35) 0%, rgba(255, 255, 255, 0.15) 50%, rgba(255, 255, 255, 0) 100%)"}}),f.jsx("span",{className:"relative z-10",children:t})]});return r?f.jsx(I.a,{href:r,className:i,style:d,whileHover:c,whileTap:u,children:h}):f.jsx(I.button,{onClick:e,className:i,style:d,whileHover:c,whileTap:u,children:h})};var w={},_e={},V={},D={},nt="Expected a function",Ie=NaN,rr="[object Symbol]",nr=/^\s+|\s+$/g,ir=/^[-+]0x[0-9a-f]+$/i,ar=/^0b[01]+$/i,or=/^0o[0-7]+$/i,sr=parseInt,lr=typeof q=="object"&&q&&q.Object===Object&&q,cr=typeof self=="object"&&self&&self.Object===Object&&self,ur=lr||cr||Function("return this")(),fr=Object.prototype,dr=fr.toString,hr=Math.max,pr=Math.min,he=function(){return ur.Date.now()};function mr(t,e,r){var n,i,s,a,o,l,c=0,u=!1,d=!1,h=!0;if(typeof t!="function")throw new TypeError(nt);e=He(e)||0,ne(r)&&(u=!!r.leading,d="maxWait"in r,s=d?hr(He(r.maxWait)||0,e):s,h="trailing"in r?!!r.trailing:h);function S(m){var k=n,H=i;return n=i=void 0,c=m,a=t.apply(H,k),a}function y(m){return c=m,o=setTimeout(v,e),u?S(m):a}function E(m){var k=m-l,H=m-c,$=e-k;return d?pr($,s-H):$}function p(m){var k=m-l,H=m-c;return l===void 0||k>=e||k<0||d&&H>=s}function v(){var m=he();if(p(m))return P(m);o=setTimeout(v,E(m))}function P(m){return o=void 0,h&&n?S(m):(n=i=void 0,a)}function B(){o!==void 0&&clearTimeout(o),c=0,n=l=i=o=void 0}function T(){return o===void 0?a:P(he())}function R(){var m=he(),k=p(m);if(n=arguments,i=this,l=m,k){if(o===void 0)return y(l);if(d)return o=setTimeout(v,e),S(l)}return o===void 0&&(o=setTimeout(v,e)),a}return R.cancel=B,R.flush=T,R}function vr(t,e,r){var n=!0,i=!0;if(typeof t!="function")throw new TypeError(nt);return ne(r)&&(n="leading"in r?!!r.leading:n,i="trailing"in r?!!r.trailing:i),mr(t,e,{leading:n,maxWait:e,trailing:i})}function ne(t){var e=typeof t;return!!t&&(e=="object"||e=="function")}function gr(t){return!!t&&typeof t=="object"}function xr(t){return typeof t=="symbol"||gr(t)&&dr.call(t)==rr}function He(t){if(typeof t=="number")return t;if(xr(t))return Ie;if(ne(t)){var e=typeof t.valueOf=="function"?t.valueOf():t;t=ne(e)?e+"":e}if(typeof t!="string")return t===0?t:+t;t=t.replace(nr,"");var r=ar.test(t);return r||or.test(t)?sr(t.slice(2),r?2:8):ir.test(t)?Ie:+t}var br=vr,N={};Object.defineProperty(N,"__esModule",{value:!0});N.addPassiveEventListener=function(e,r,n){var i=n.name;i||(i=r,console.warn("Listener must be a named function.")),re.has(r)||re.set(r,new Set);var s=re.get(r);if(!s.has(i)){var a=function(){var o=!1;try{var l=Object.defineProperty({},"passive",{get:function(){o=!0}});window.addEventListener("test",null,l)}catch{}return o}();e.addEventListener(r,n,a?{passive:!0}:!1),s.add(i)}};N.removePassiveEventListener=function(e,r,n){e.removeEventListener(r,n),re.get(r).delete(n.name||r)};var re=new Map;Object.defineProperty(D,"__esModule",{value:!0});var _r=br,yr=wr(_r),Le=N;function wr(t){return t&&t.__esModule?t:{default:t}}var Sr=function(e){var r=arguments.length>1&&arguments[1]!==void 0?arguments[1]:66;return(0,yr.default)(e,r)},b={spyCallbacks:[],spySetState:[],scrollSpyContainers:[],mount:function(e,r){if(e){var n=Sr(function(i){b.scrollHandler(e)},r);return b.scrollSpyContainers.push(e),(0,Le.addPassiveEventListener)(e,"scroll",n),function(){(0,Le.removePassiveEventListener)(e,"scroll",n),b.scrollSpyContainers.splice(b.scrollSpyContainers.indexOf(e),1)}}return function(){}},isMounted:function(e){return b.scrollSpyContainers.indexOf(e)!==-1},currentPositionX:function(e){if(e===document){var r=window.scrollY!==void 0,n=(document.compatMode||"")==="CSS1Compat";return r?window.scrollX:n?document.documentElement.scrollLeft:document.body.scrollLeft}else return e.scrollLeft},currentPositionY:function(e){if(e===document){var r=window.scrollX!==void 0,n=(document.compatMode||"")==="CSS1Compat";return r?window.scrollY:n?document.documentElement.scrollTop:document.body.scrollTop}else return e.scrollTop},scrollHandler:function(e){var r=b.scrollSpyContainers[b.scrollSpyContainers.indexOf(e)].spyCallbacks||[];r.forEach(function(n){return n(b.currentPositionX(e),b.currentPositionY(e))})},addStateHandler:function(e){b.spySetState.push(e)},addSpyHandler:function(e,r){var n=b.scrollSpyContainers[b.scrollSpyContainers.indexOf(r)];n.spyCallbacks||(n.spyCallbacks=[]),n.spyCallbacks.push(e)},updateStates:function(){b.spySetState.forEach(function(e){return e()})},unmount:function(e,r){b.scrollSpyContainers.forEach(function(n){return n.spyCallbacks&&n.spyCallbacks.length&&n.spyCallbacks.indexOf(r)>-1&&n.spyCallbacks.splice(n.spyCallbacks.indexOf(r),1)}),b.spySetState&&b.spySetState.length&&b.spySetState.indexOf(e)>-1&&b.spySetState.splice(b.spySetState.indexOf(e),1),document.removeEventListener("scroll",b.scrollHandler)},update:function(){return b.scrollSpyContainers.forEach(function(e){return b.scrollHandler(e)})}};D.default=b;var C={},W={};Object.defineProperty(W,"__esModule",{value:!0});var Er=function(e,r){var n=e.indexOf("#")===0?e.substring(1):e,i=n?"#"+n:"",s=window&&window.location,a=i?s.pathname+s.search+i:s.pathname+s.search;r?history.pushState(history.state,"",a):history.replaceState(history.state,"",a)},Or=function(){return window.location.hash.replace(/^#/,"")},Tr=function(e){return function(r){return e.contains?e!=r&&e.contains(r):!!(e.compareDocumentPosition(r)&16)}},Rr=function(e){return getComputedStyle(e).position!=="static"},pe=function(e,r){for(var n=e.offsetTop,i=e.offsetParent;i&&!r(i);)n+=i.offsetTop,i=i.offsetParent;return{offsetTop:n,offsetParent:i}},kr=function(e,r,n){if(n)return e===document?r.getBoundingClientRect().left+(window.scrollX||window.pageXOffset):getComputedStyle(e).position!=="static"?r.offsetLeft:r.offsetLeft-e.offsetLeft;if(e===document)return r.getBoundingClientRect().top+(window.scrollY||window.pageYOffset);if(Rr(e)){if(r.offsetParent!==e){var i=function(u){return u===e||u===document},s=pe(r,i),a=s.offsetTop,o=s.offsetParent;if(o!==e)throw new Error("Seems containerElement is not an ancestor of the Element");return a}return r.offsetTop}if(r.offsetParent===e.offsetParent)return r.offsetTop-e.offsetTop;var l=function(u){return u===document};return pe(r,l).offsetTop-pe(e,l).offsetTop};W.default={updateHash:Er,getHash:Or,filterElementInContainer:Tr,scrollOffset:kr};var ae={},ye={};Object.defineProperty(ye,"__esModule",{value:!0});ye.default={defaultEasing:function(e){return e<.5?Math.pow(e*2,2)/2:1-Math.pow((1-e)*2,2)/2},linear:function(e){return e},easeInQuad:function(e){return e*e},easeOutQuad:function(e){return e*(2-e)},easeInOutQuad:function(e){return e<.5?2*e*e:-1+(4-2*e)*e},easeInCubic:function(e){return e*e*e},easeOutCubic:function(e){return--e*e*e+1},easeInOutCubic:function(e){return e<.5?4*e*e*e:(e-1)*(2*e-2)*(2*e-2)+1},easeInQuart:function(e){return e*e*e*e},easeOutQuart:function(e){return 1- --e*e*e*e},easeInOutQuart:function(e){return e<.5?8*e*e*e*e:1-8*--e*e*e*e},easeInQuint:function(e){return e*e*e*e*e},easeOutQuint:function(e){return 1+--e*e*e*e*e},easeInOutQuint:function(e){return e<.5?16*e*e*e*e*e:1+16*--e*e*e*e*e}};var we={};Object.defineProperty(we,"__esModule",{value:!0});var Pr=N,jr=["mousedown","wheel","touchmove","keydown"];we.default={subscribe:function(e){return typeof document<"u"&&jr.forEach(function(r){return(0,Pr.addPassiveEventListener)(document,r,e)})}};var F={};Object.defineProperty(F,"__esModule",{value:!0});var xe={registered:{},scrollEvent:{register:function(e,r){xe.registered[e]=r},remove:function(e){xe.registered[e]=null}}};F.default=xe;Object.defineProperty(ae,"__esModule",{value:!0});var Ur=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},Br=W;oe(Br);var Mr=ye,ze=oe(Mr),Ar=we,Ir=oe(Ar),Hr=F,j=oe(Hr);function oe(t){return t&&t.__esModule?t:{default:t}}var it=function(e){return ze.default[e.smooth]||ze.default.defaultEasing},Lr=function(e){return typeof e=="function"?e:function(){return e}},zr=function(){if(typeof window<"u")return window.requestAnimationFrame||window.webkitRequestAnimationFrame},be=function(){return zr()||function(t,e,r){window.setTimeout(t,r||1e3/60,new Date().getTime())}}(),at=function(){return{currentPosition:0,startPosition:0,targetPosition:0,progress:0,duration:0,cancel:!1,target:null,containerElement:null,to:null,start:null,delta:null,percent:null,delayTimeout:null}},ot=function(e){var r=e.data.containerElement;if(r&&r!==document&&r!==document.body)return r.scrollLeft;var n=window.pageXOffset!==void 0,i=(document.compatMode||"")==="CSS1Compat";return n?window.pageXOffset:i?document.documentElement.scrollLeft:document.body.scrollLeft},st=function(e){var r=e.data.containerElement;if(r&&r!==document&&r!==document.body)return r.scrollTop;var n=window.pageXOffset!==void 0,i=(document.compatMode||"")==="CSS1Compat";return n?window.pageYOffset:i?document.documentElement.scrollTop:document.body.scrollTop},Cr=function(e){var r=e.data.containerElement;if(r&&r!==document&&r!==document.body)return r.scrollWidth-r.offsetWidth;var n=document.body,i=document.documentElement;return Math.max(n.scrollWidth,n.offsetWidth,i.clientWidth,i.scrollWidth,i.offsetWidth)},$r=function(e){var r=e.data.containerElement;if(r&&r!==document&&r!==document.body)return r.scrollHeight-r.offsetHeight;var n=document.body,i=document.documentElement;return Math.max(n.scrollHeight,n.offsetHeight,i.clientHeight,i.scrollHeight,i.offsetHeight)},Vr=function t(e,r,n){var i=r.data;if(!r.ignoreCancelEvents&&i.cancel){j.default.registered.end&&j.default.registered.end(i.to,i.target,i.currentPositionY);return}if(i.delta=Math.round(i.targetPosition-i.startPosition),i.start===null&&(i.start=n),i.progress=n-i.start,i.percent=i.progress>=i.duration?1:e(i.progress/i.duration),i.currentPosition=i.startPosition+Math.ceil(i.delta*i.percent),i.containerElement&&i.containerElement!==document&&i.containerElement!==document.body?r.horizontal?i.containerElement.scrollLeft=i.currentPosition:i.containerElement.scrollTop=i.currentPosition:r.horizontal?window.scrollTo(i.currentPosition,0):window.scrollTo(0,i.currentPosition),i.percent<1){var s=t.bind(null,e,r);be.call(window,s);return}j.default.registered.end&&j.default.registered.end(i.to,i.target,i.currentPosition)},Se=function(e){e.data.containerElement=e?e.containerId?document.getElementById(e.containerId):e.container&&e.container.nodeType?e.container:document:null},X=function(e,r,n,i){r.data=r.data||at(),window.clearTimeout(r.data.delayTimeout);var s=function(){r.data.cancel=!0};if(Ir.default.subscribe(s),Se(r),r.data.start=null,r.data.cancel=!1,r.data.startPosition=r.horizontal?ot(r):st(r),r.data.targetPosition=r.absolute?e:e+r.data.startPosition,r.data.startPosition===r.data.targetPosition){j.default.registered.end&&j.default.registered.end(r.data.to,r.data.target,r.data.currentPosition);return}r.data.delta=Math.round(r.data.targetPosition-r.data.startPosition),r.data.duration=Lr(r.duration)(r.data.delta),r.data.duration=isNaN(parseFloat(r.data.duration))?1e3:parseFloat(r.data.duration),r.data.to=n,r.data.target=i;var a=it(r),o=Vr.bind(null,a,r);if(r&&r.delay>0){r.data.delayTimeout=window.setTimeout(function(){j.default.registered.begin&&j.default.registered.begin(r.data.to,r.data.target),be.call(window,o)},r.delay);return}j.default.registered.begin&&j.default.registered.begin(r.data.to,r.data.target),be.call(window,o)},se=function(e){return e=Ur({},e),e.data=e.data||at(),e.absolute=!0,e},Dr=function(e){X(0,se(e))},Nr=function(e,r){X(e,se(r))},Wr=function(e){e=se(e),Se(e),X(e.horizontal?Cr(e):$r(e),e)},Fr=function(e,r){r=se(r),Se(r);var n=r.horizontal?ot(r):st(r);X(e+n,r)};ae.default={animateTopScroll:X,getAnimationType:it,scrollToTop:Dr,scrollToBottom:Wr,scrollTo:Nr,scrollMore:Fr};Object.defineProperty(C,"__esModule",{value:!0});var Xr=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},Gr=W,Yr=Ee(Gr),qr=ae,Qr=Ee(qr),Kr=F,K=Ee(Kr);function Ee(t){return t&&t.__esModule?t:{default:t}}var Z={},Ce=void 0;C.default={unmount:function(){Z={}},register:function(e,r){Z[e]=r},unregister:function(e){delete Z[e]},get:function(e){return Z[e]||document.getElementById(e)||document.getElementsByName(e)[0]||document.getElementsByClassName(e)[0]},setActiveLink:function(e){return Ce=e},getActiveLink:function(){return Ce},scrollTo:function(e,r){var n=this.get(e);if(!n){console.warn("target Element not found");return}r=Xr({},r,{absolute:!1});var i=r.containerId,s=r.container,a=void 0;i?a=document.getElementById(i):s&&s.nodeType?a=s:a=document,r.absolute=!0;var o=r.horizontal,l=Yr.default.scrollOffset(a,n,o)+(r.offset||0);if(!r.smooth){K.default.registered.begin&&K.default.registered.begin(e,n),a===document?r.horizontal?window.scrollTo(l,0):window.scrollTo(0,l):a.scrollTop=l,K.default.registered.end&&K.default.registered.end(e,n);return}Qr.default.animateTopScroll(l,r,e,n)}};var le={};Object.defineProperty(le,"__esModule",{value:!0});var Zr=W,me=Jr(Zr);function Jr(t){return t&&t.__esModule?t:{default:t}}var en={mountFlag:!1,initialized:!1,scroller:null,containers:{},mount:function(e){this.scroller=e,this.handleHashChange=this.handleHashChange.bind(this),window.addEventListener("hashchange",this.handleHashChange),this.initStateFromHash(),this.mountFlag=!0},mapContainer:function(e,r){this.containers[e]=r},isMounted:function(){return this.mountFlag},isInitialized:function(){return this.initialized},initStateFromHash:function(){var e=this,r=this.getHash();r?window.setTimeout(function(){e.scrollTo(r,!0),e.initialized=!0},10):this.initialized=!0},scrollTo:function(e,r){var n=this.scroller,i=n.get(e);if(i&&(r||e!==n.getActiveLink())){var s=this.containers[e]||document;n.scrollTo(e,{container:s})}},getHash:function(){return me.default.getHash()},changeHash:function(e,r){this.isInitialized()&&me.default.getHash()!==e&&me.default.updateHash(e,r)},handleHashChange:function(){this.scrollTo(this.getHash())},unmount:function(){this.scroller=null,this.containers=null,window.removeEventListener("hashchange",this.handleHashChange)}};le.default=en;Object.defineProperty(V,"__esModule",{value:!0});var J=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},tn=function(){function t(e,r){for(var n=0;n<r.length;n++){var i=r[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}return function(e,r,n){return r&&t(e.prototype,r),n&&t(e,n),e}}(),rn=x,$e=G(rn),nn=D,ee=G(nn),an=C,on=G(an),sn=ie,g=G(sn),ln=le,M=G(ln);function G(t){return t&&t.__esModule?t:{default:t}}function cn(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function un(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function fn(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}var Ve={to:g.default.string.isRequired,containerId:g.default.string,container:g.default.object,activeClass:g.default.string,activeStyle:g.default.object,spy:g.default.bool,horizontal:g.default.bool,smooth:g.default.oneOfType([g.default.bool,g.default.string]),offset:g.default.number,delay:g.default.number,isDynamic:g.default.bool,onClick:g.default.func,duration:g.default.oneOfType([g.default.number,g.default.func]),absolute:g.default.bool,onSetActive:g.default.func,onSetInactive:g.default.func,ignoreCancelEvents:g.default.bool,hashSpy:g.default.bool,saveHashHistory:g.default.bool,spyThrottle:g.default.number};V.default=function(t,e){var r=e||on.default,n=function(s){fn(a,s);function a(o){cn(this,a);var l=un(this,(a.__proto__||Object.getPrototypeOf(a)).call(this,o));return i.call(l),l.state={active:!1},l.beforeUnmountCallbacks=[],l}return tn(a,[{key:"getScrollSpyContainer",value:function(){var l=this.props.containerId,c=this.props.container;return l&&!c?document.getElementById(l):c&&c.nodeType?c:document}},{key:"componentDidMount",value:function(){if(this.props.spy||this.props.hashSpy){var l=this.getScrollSpyContainer();if(!ee.default.isMounted(l)){var c=ee.default.mount(l,this.props.spyThrottle);this.beforeUnmountCallbacks.push(c)}this.props.hashSpy&&(M.default.isMounted()||M.default.mount(r),M.default.mapContainer(this.props.to,l)),ee.default.addSpyHandler(this.spyHandler,l),this.setState({container:l})}}},{key:"componentWillUnmount",value:function(){ee.default.unmount(this.stateHandler,this.spyHandler),this.beforeUnmountCallbacks.forEach(function(l){return l()})}},{key:"render",value:function(){var l="";this.state&&this.state.active?l=((this.props.className||"")+" "+(this.props.activeClass||"active")).trim():l=this.props.className;var c={};this.state&&this.state.active?c=J({},this.props.style,this.props.activeStyle):c=J({},this.props.style);var u=J({},this.props);for(var d in Ve)u.hasOwnProperty(d)&&delete u[d];return u.className=l,u.style=c,u.onClick=this.handleClick,$e.default.createElement(t,u)}}]),a}($e.default.PureComponent),i=function(){var a=this;this.scrollTo=function(o,l){r.scrollTo(o,J({},a.state,l))},this.handleClick=function(o){a.props.onClick&&a.props.onClick(o),o.stopPropagation&&o.stopPropagation(),o.preventDefault&&o.preventDefault(),a.scrollTo(a.props.to,a.props)},this.spyHandler=function(o,l){var c=a.getScrollSpyContainer();if(!(M.default.isMounted()&&!M.default.isInitialized())){var u=a.props.horizontal,d=a.props.to,h=null,S=void 0,y=void 0;if(u){var E=0,p=0,v=0;if(c.getBoundingClientRect){var P=c.getBoundingClientRect();v=P.left}if(!h||a.props.isDynamic){if(h=r.get(d),!h)return;var B=h.getBoundingClientRect();E=B.left-v+o,p=E+B.width}var T=o-a.props.offset;S=T>=Math.floor(E)&&T<Math.floor(p),y=T<Math.floor(E)||T>=Math.floor(p)}else{var R=0,m=0,k=0;if(c.getBoundingClientRect){var H=c.getBoundingClientRect();k=H.top}if(!h||a.props.isDynamic){if(h=r.get(d),!h)return;var $=h.getBoundingClientRect();R=$.top-k+l,m=R+$.height}var Y=l-a.props.offset;S=Y>=Math.floor(R)&&Y<Math.floor(m),y=Y<Math.floor(R)||Y>=Math.floor(m)}var ke=r.getActiveLink();if(y){if(d===ke&&r.setActiveLink(void 0),a.props.hashSpy&&M.default.getHash()===d){var Pe=a.props.saveHashHistory,wt=Pe===void 0?!1:Pe;M.default.changeHash("",wt)}a.props.spy&&a.state.active&&(a.setState({active:!1}),a.props.onSetInactive&&a.props.onSetInactive(d,h))}if(S&&(ke!==d||a.state.active===!1)){r.setActiveLink(d);var je=a.props.saveHashHistory,St=je===void 0?!1:je;a.props.hashSpy&&M.default.changeHash(d,St),a.props.spy&&(a.setState({active:!0}),a.props.onSetActive&&a.props.onSetActive(d,h))}}}};return n.propTypes=Ve,n.defaultProps={offset:0},n};Object.defineProperty(_e,"__esModule",{value:!0});var dn=x,De=lt(dn),hn=V,pn=lt(hn);function lt(t){return t&&t.__esModule?t:{default:t}}function mn(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function Ne(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function vn(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}var gn=function(t){vn(e,t);function e(){var r,n,i,s;mn(this,e);for(var a=arguments.length,o=Array(a),l=0;l<a;l++)o[l]=arguments[l];return s=(n=(i=Ne(this,(r=e.__proto__||Object.getPrototypeOf(e)).call.apply(r,[this].concat(o))),i),i.render=function(){return De.default.createElement("a",i.props,i.props.children)},n),Ne(i,s)}return e}(De.default.Component);_e.default=(0,pn.default)(gn);var Oe={};Object.defineProperty(Oe,"__esModule",{value:!0});var xn=function(){function t(e,r){for(var n=0;n<r.length;n++){var i=r[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}return function(e,r,n){return r&&t(e.prototype,r),n&&t(e,n),e}}(),bn=x,We=ct(bn),_n=V,yn=ct(_n);function ct(t){return t&&t.__esModule?t:{default:t}}function wn(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function Sn(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function En(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}var On=function(t){En(e,t);function e(){return wn(this,e),Sn(this,(e.__proto__||Object.getPrototypeOf(e)).apply(this,arguments))}return xn(e,[{key:"render",value:function(){return We.default.createElement("button",this.props,this.props.children)}}]),e}(We.default.Component);Oe.default=(0,yn.default)(On);var Te={},ce={};Object.defineProperty(ce,"__esModule",{value:!0});var Tn=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},Rn=function(){function t(e,r){for(var n=0;n<r.length;n++){var i=r[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}return function(e,r,n){return r&&t(e.prototype,r),n&&t(e,n),e}}(),kn=x,Fe=ue(kn),Pn=Ot;ue(Pn);var jn=C,Xe=ue(jn),Un=ie,Ge=ue(Un);function ue(t){return t&&t.__esModule?t:{default:t}}function Bn(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function Mn(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function An(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}ce.default=function(t){var e=function(r){An(n,r);function n(i){Bn(this,n);var s=Mn(this,(n.__proto__||Object.getPrototypeOf(n)).call(this,i));return s.childBindings={domNode:null},s}return Rn(n,[{key:"componentDidMount",value:function(){if(typeof window>"u")return!1;this.registerElems(this.props.name)}},{key:"componentDidUpdate",value:function(s){this.props.name!==s.name&&this.registerElems(this.props.name)}},{key:"componentWillUnmount",value:function(){if(typeof window>"u")return!1;Xe.default.unregister(this.props.name)}},{key:"registerElems",value:function(s){Xe.default.register(s,this.childBindings.domNode)}},{key:"render",value:function(){return Fe.default.createElement(t,Tn({},this.props,{parentBindings:this.childBindings}))}}]),n}(Fe.default.Component);return e.propTypes={name:Ge.default.string,id:Ge.default.string},e};Object.defineProperty(Te,"__esModule",{value:!0});var Ye=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},In=function(){function t(e,r){for(var n=0;n<r.length;n++){var i=r[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}return function(e,r,n){return r&&t(e.prototype,r),n&&t(e,n),e}}(),Hn=x,qe=Re(Hn),Ln=ce,zn=Re(Ln),Cn=ie,Qe=Re(Cn);function Re(t){return t&&t.__esModule?t:{default:t}}function $n(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function Vn(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function Dn(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}var ut=function(t){Dn(e,t);function e(){return $n(this,e),Vn(this,(e.__proto__||Object.getPrototypeOf(e)).apply(this,arguments))}return In(e,[{key:"render",value:function(){var n=this,i=Ye({},this.props);return delete i.name,i.parentBindings&&delete i.parentBindings,qe.default.createElement("div",Ye({},i,{ref:function(a){n.props.parentBindings.domNode=a}}),this.props.children)}}]),e}(qe.default.Component);ut.propTypes={name:Qe.default.string,id:Qe.default.string};Te.default=(0,zn.default)(ut);var ve=Object.assign||function(t){for(var e=1;e<arguments.length;e++){var r=arguments[e];for(var n in r)Object.prototype.hasOwnProperty.call(r,n)&&(t[n]=r[n])}return t},Ke=function(){function t(e,r){for(var n=0;n<r.length;n++){var i=r[n];i.enumerable=i.enumerable||!1,i.configurable=!0,"value"in i&&(i.writable=!0),Object.defineProperty(e,i.key,i)}}return function(e,r,n){return r&&t(e.prototype,r),n&&t(e,n),e}}();function Ze(t,e){if(!(t instanceof e))throw new TypeError("Cannot call a class as a function")}function Je(t,e){if(!t)throw new ReferenceError("this hasn't been initialised - super() hasn't been called");return e&&(typeof e=="object"||typeof e=="function")?e:t}function et(t,e){if(typeof e!="function"&&e!==null)throw new TypeError("Super expression must either be null or a function, not "+typeof e);t.prototype=Object.create(e&&e.prototype,{constructor:{value:t,enumerable:!1,writable:!0,configurable:!0}}),e&&(Object.setPrototypeOf?Object.setPrototypeOf(t,e):t.__proto__=e)}var te=x,L=D,ge=C,_=ie,A=le,tt={to:_.string.isRequired,containerId:_.string,container:_.object,activeClass:_.string,spy:_.bool,smooth:_.oneOfType([_.bool,_.string]),offset:_.number,delay:_.number,isDynamic:_.bool,onClick:_.func,duration:_.oneOfType([_.number,_.func]),absolute:_.bool,onSetActive:_.func,onSetInactive:_.func,ignoreCancelEvents:_.bool,hashSpy:_.bool,spyThrottle:_.number},Nn={Scroll:function(e,r){console.warn("Helpers.Scroll is deprecated since v1.7.0");var n=r||ge,i=function(a){et(o,a);function o(l){Ze(this,o);var c=Je(this,(o.__proto__||Object.getPrototypeOf(o)).call(this,l));return s.call(c),c.state={active:!1},c}return Ke(o,[{key:"getScrollSpyContainer",value:function(){var c=this.props.containerId,u=this.props.container;return c?document.getElementById(c):u&&u.nodeType?u:document}},{key:"componentDidMount",value:function(){if(this.props.spy||this.props.hashSpy){var c=this.getScrollSpyContainer();L.isMounted(c)||L.mount(c,this.props.spyThrottle),this.props.hashSpy&&(A.isMounted()||A.mount(n),A.mapContainer(this.props.to,c)),this.props.spy&&L.addStateHandler(this.stateHandler),L.addSpyHandler(this.spyHandler,c),this.setState({container:c})}}},{key:"componentWillUnmount",value:function(){L.unmount(this.stateHandler,this.spyHandler)}},{key:"render",value:function(){var c="";this.state&&this.state.active?c=((this.props.className||"")+" "+(this.props.activeClass||"active")).trim():c=this.props.className;var u=ve({},this.props);for(var d in tt)u.hasOwnProperty(d)&&delete u[d];return u.className=c,u.onClick=this.handleClick,te.createElement(e,u)}}]),o}(te.Component),s=function(){var o=this;this.scrollTo=function(l,c){n.scrollTo(l,ve({},o.state,c))},this.handleClick=function(l){o.props.onClick&&o.props.onClick(l),l.stopPropagation&&l.stopPropagation(),l.preventDefault&&l.preventDefault(),o.scrollTo(o.props.to,o.props)},this.stateHandler=function(){n.getActiveLink()!==o.props.to&&(o.state!==null&&o.state.active&&o.props.onSetInactive&&o.props.onSetInactive(),o.setState({active:!1}))},this.spyHandler=function(l){var c=o.getScrollSpyContainer();if(!(A.isMounted()&&!A.isInitialized())){var u=o.props.to,d=null,h=0,S=0,y=0;if(c.getBoundingClientRect){var E=c.getBoundingClientRect();y=E.top}if(!d||o.props.isDynamic){if(d=n.get(u),!d)return;var p=d.getBoundingClientRect();h=p.top-y+l,S=h+p.height}var v=l-o.props.offset,P=v>=Math.floor(h)&&v<Math.floor(S),B=v<Math.floor(h)||v>=Math.floor(S),T=n.getActiveLink();if(B)return u===T&&n.setActiveLink(void 0),o.props.hashSpy&&A.getHash()===u&&A.changeHash(),o.props.spy&&o.state.active&&(o.setState({active:!1}),o.props.onSetInactive&&o.props.onSetInactive()),L.updateStates();if(P&&T!==u)return n.setActiveLink(u),o.props.hashSpy&&A.changeHash(u),o.props.spy&&(o.setState({active:!0}),o.props.onSetActive&&o.props.onSetActive(u)),L.updateStates()}}};return i.propTypes=tt,i.defaultProps={offset:0},i},Element:function(e){console.warn("Helpers.Element is deprecated since v1.7.0");var r=function(n){et(i,n);function i(s){Ze(this,i);var a=Je(this,(i.__proto__||Object.getPrototypeOf(i)).call(this,s));return a.childBindings={domNode:null},a}return Ke(i,[{key:"componentDidMount",value:function(){if(typeof window>"u")return!1;this.registerElems(this.props.name)}},{key:"componentDidUpdate",value:function(a){this.props.name!==a.name&&this.registerElems(this.props.name)}},{key:"componentWillUnmount",value:function(){if(typeof window>"u")return!1;ge.unregister(this.props.name)}},{key:"registerElems",value:function(a){ge.register(a,this.childBindings.domNode)}},{key:"render",value:function(){return te.createElement(e,ve({},this.props,{parentBindings:this.childBindings}))}}]),i}(te.Component);return r.propTypes={name:_.string,id:_.string},r}},Wn=Nn;Object.defineProperty(w,"__esModule",{value:!0});w.Helpers=w.ScrollElement=w.ScrollLink=w.animateScroll=w.scrollSpy=w.Events=w.scroller=w.Element=w.Button=yt=w.Link=void 0;var Fn=_e,ft=U(Fn),Xn=Oe,dt=U(Xn),Gn=Te,ht=U(Gn),Yn=C,pt=U(Yn),qn=F,mt=U(qn),Qn=D,vt=U(Qn),Kn=ae,gt=U(Kn),Zn=V,xt=U(Zn),Jn=ce,bt=U(Jn),ei=Wn,_t=U(ei);function U(t){return t&&t.__esModule?t:{default:t}}var yt=w.Link=ft.default;w.Button=dt.default;w.Element=ht.default;w.scroller=pt.default;w.Events=mt.default;w.scrollSpy=vt.default;w.animateScroll=gt.default;w.ScrollLink=xt.default;w.ScrollElement=bt.default;w.Helpers=_t.default;w.default={Link:ft.default,Button:dt.default,Element:ht.default,scroller:pt.default,Events:mt.default,scrollSpy:vt.default,animateScroll:gt.default,ScrollLink:xt.default,ScrollElement:bt.default,Helpers:_t.default};function z({children:t,delay:e=0}){const r=x.useRef(null),n=tr(r,{once:!0,amount:.3});return f.jsx(I.div,{ref:r,initial:{opacity:0,y:50},animate:n?{opacity:1,y:0}:{opacity:0,y:50},transition:{duration:.6,delay:e},children:t})}function ti(){return f.jsxs("div",{className:"relative",children:[f.jsx("div",{className:"fixed top-0 left-0 w-full h-full -z-10",children:f.jsx(Zt,{className:"w-full h-full",colors:["#493b7c","#604c9c","#9e8cb4","#dfc9ad"],speed:.5,distortion:.5,swirl:.3,grainMixer:.2,grainOverlay:.1})}),f.jsxs("section",{className:"relative min-h-screen flex flex-col justify-center items-center px-10 py-32",children:[f.jsx(I.div,{initial:{opacity:0,y:30},animate:{opacity:1,y:0},transition:{duration:.8},className:"text-6xl font-black tracking-tighter mb-6 text-center text-black dark:text-white",children:"CONTINUUM"}),f.jsx(I.h1,{initial:{opacity:0,y:30},animate:{opacity:1,y:0},transition:{duration:.8,delay:.2},className:"text-3xl font-light text-center max-w-3xl mb-5 leading-snug text-black dark:text-white",children:"The First Skill-Based Wagering Platform for Real-World Golf"}),f.jsx(I.p,{initial:{opacity:0,y:30},animate:{opacity:1,y:0},transition:{duration:.8,delay:.4},className:"text-lg text-center max-w-2xl mb-12 leading-relaxed text-black dark:text-white",children:"Continuum Technologies is building the software platform that transforms any golf simulator into a competitive, real-money gameplay experience. Integrated with Trackman, Foresight, and Uneekor systems, we're creating an entirely new market at the intersection of gaming, golf, and sports wagering."}),f.jsxs(I.div,{initial:{opacity:0,y:30},animate:{opacity:1,y:0},transition:{duration:.8,delay:.6},className:"flex gap-5 mb-16 flex-wrap justify-center",children:[f.jsx(de,{href:"/app.html",variant:"secondary",children:"Try Live Demo"}),f.jsx(yt,{to:"how-it-works",smooth:!0,duration:800,offset:-50,children:f.jsx(de,{variant:"secondary",children:"Learn More"})})]}),f.jsxs(I.div,{initial:{opacity:0,y:30},animate:{opacity:1,y:0},transition:{duration:.8,delay:.8},className:"flex gap-16 flex-wrap justify-center",children:[f.jsxs("div",{className:"text-center",children:[f.jsx("div",{className:"text-5xl font-black leading-none mb-2 text-black dark:text-white",children:"$115B+"}),f.jsx("div",{className:"text-sm uppercase tracking-wider font-semibold text-black dark:text-white",children:"Sports Betting Market"})]}),f.jsxs("div",{className:"text-center",children:[f.jsx("div",{className:"text-5xl font-black leading-none mb-2 text-black dark:text-white",children:"20M+"}),f.jsx("div",{className:"text-sm uppercase tracking-wider font-semibold text-black dark:text-white",children:"Golfers in US"})]}),f.jsxs("div",{className:"text-center",children:[f.jsx("div",{className:"text-5xl font-black leading-none mb-2 text-black dark:text-white",children:"New"}),f.jsx("div",{className:"text-sm uppercase tracking-wider font-semibold text-black dark:text-white",children:"Market Category"})]})]})]}),f.jsxs("section",{className:"relative px-10 py-32",children:[f.jsxs(z,{children:[f.jsx("h2",{className:"text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white",children:"What We're Building"}),f.jsx("p",{className:"text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white",children:"The first-ever platform to unlock skill-based sports wagering for a real-world, physical sport with measurable outcomes."})]}),f.jsx("div",{className:"grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-10 max-w-6xl mx-auto",children:[{title:"Mobile App for Players",description:"Enter challenges, track performance, and compete for real money. Personalized payout curves adapt to each golfer's accuracy and consistency—fair for every skill level."},{title:"Simulator Integration",description:"Direct integration with Trackman, Foresight, and Uneekor. Real-time shot data feeds into our backend to generate instant, skill-based payouts."},{title:"Venue Backend Portal",description:"Operators can configure contests, monitor gameplay, and track revenue in real-time. Turn every bay into a competitive gaming experience."},{title:"Dynamic & Fair",description:"Every player gets a personalized experience. Better golfers see higher payouts, while beginners get accessible entry points. It's skill-based, not luck-based."},{title:"New Revenue Streams",description:"Drive engagement and generate income for simulator venues. Players stay longer, compete more, and return for the competitive experience."},{title:"Creating a New Market",description:"Not just riding the sports betting wave—we're opening up an entirely new category at the intersection of golf, gaming, and immersive tech."}].map((t,e)=>f.jsx(z,{delay:e*.1,children:f.jsxs("div",{className:"bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-3xl p-10 hover:border-white/50 hover:bg-white/30 transition-all h-full flex flex-col",children:[f.jsx("h3",{className:"text-2xl font-bold mb-3 text-black dark:text-white",children:t.title}),f.jsx("p",{className:"text-base leading-relaxed text-black dark:text-white flex-grow",children:t.description})]})},e))})]}),f.jsxs("section",{className:"relative px-10 py-32",id:"how-it-works",children:[f.jsxs(z,{children:[f.jsx("h2",{className:"text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white",children:"How It Works"}),f.jsx("p",{className:"text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white",children:"From simulator to payout in seconds—here's the player experience."})]}),f.jsx("div",{className:"max-w-4xl mx-auto",children:[{num:1,title:"Open the App & Enter a Challenge",description:"Players open our mobile app, select a challenge, and place their wager. They can compete solo or against others in multiplayer formats."},{num:2,title:"Take Your Shot",description:"Hit the ball on a Trackman, Foresight, or Uneekor simulator. Our platform captures real-time shot data—distance, accuracy, and consistency."},{num:3,title:"Skill-Based Payout Generated",description:"Our backend instantly calculates a personalized payout based on shot accuracy relative to your skill profile. Fair, dynamic, and tailored to you."},{num:4,title:"Win & Track Your Progress",description:"Receive your payout immediately. Track your performance over time, compare with others, and climb the leaderboard."}].map((t,e)=>f.jsx(z,{delay:e*.15,children:f.jsx("div",{className:"bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-3xl p-10 mb-10 hover:border-white/50 hover:bg-white/30 transition-all",children:f.jsxs("div",{className:"flex items-start gap-6",children:[f.jsx("div",{className:"min-w-[60px] h-16 flex items-center justify-center text-3xl font-black text-black dark:text-white",children:t.num}),f.jsxs("div",{className:"flex-1",children:[f.jsx("h3",{className:"text-3xl font-bold mb-3 text-black dark:text-white",children:t.title}),f.jsx("p",{className:"text-base leading-relaxed text-black dark:text-white",children:t.description})]})]})})},e))})]}),f.jsxs("section",{className:"relative px-10 py-32",children:[f.jsxs(z,{children:[f.jsx("h2",{className:"text-5xl font-extrabold text-center mb-5 tracking-tight text-black dark:text-white",children:"Why This Is Unique"}),f.jsx("p",{className:"text-lg text-center max-w-2xl mx-auto mb-20 leading-relaxed text-black dark:text-white",children:"This is the first time skill-based sports wagering has been unlocked for a real-world, physical sport with measurable outcomes."})]}),f.jsx("div",{className:"grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-8 max-w-6xl mx-auto",children:[{label:"Market",value:"First-of-Its-Kind"},{label:"Integration",value:"Major Simulators"},{label:"Payouts",value:"Personalized & Fair"},{label:"Revenue",value:"New Streams for Venues"}].map((t,e)=>f.jsx(z,{delay:e*.1,children:f.jsxs("div",{className:"bg-gradient-to-br from-white/40 to-white/20 backdrop-blur-2xl border border-white/30 rounded-2xl p-8 text-center hover:border-white/50 hover:bg-white/30 transition-all h-full flex flex-col justify-center min-h-[140px]",children:[f.jsx("div",{className:"text-sm uppercase tracking-wider mb-2 font-semibold text-black dark:text-white",children:t.label}),f.jsx("div",{className:"text-2xl font-extrabold text-black dark:text-white",children:t.value})]})},e))})]}),f.jsx("section",{className:"relative px-10 py-32 text-center",children:f.jsxs(z,{children:[f.jsx("h2",{className:"text-6xl font-black mb-6 tracking-tighter text-black dark:text-white",children:"See the Platform in Action"}),f.jsx("p",{className:"text-xl mb-10 max-w-2xl mx-auto text-black dark:text-white",children:"Explore our interactive demo showcasing real-time gameplay simulation, venue analytics, and dynamic payout calculations."}),f.jsx("div",{className:"flex gap-5 justify-center",children:f.jsx(de,{href:"/app.html",variant:"secondary",children:"Launch Interactive Demo"})})]})}),f.jsxs("footer",{className:"relative px-10 py-16 text-center",children:[f.jsx("div",{className:"text-3xl font-black mb-5 text-black dark:text-white",children:"CONTINUUM"}),f.jsx("p",{className:"text-sm mb-3 text-black dark:text-white",children:"Creating a New Market at the Intersection of Golf, Gaming, and Sports Wagering"}),f.jsx("p",{className:"text-sm mb-3 text-black dark:text-white",children:"Continuum Technologies"}),f.jsx("p",{className:"text-xs mt-5 text-black/60 dark:text-white/60",children:"© 2024 Continuum Technologies. All rights reserved."})]})]})}Tt.createRoot(document.getElementById("root")).render(f.jsx(Rt.StrictMode,{children:f.jsx(ti,{})}));
