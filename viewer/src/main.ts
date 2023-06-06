import "./style.css";
import { GLWindow } from "./glwindow";
import data from "./data.bin?raw";
import tz1DataStr from "./tz1s.txt?raw";
import verifiedIdsStr from "./verified_ids.csv?raw";
import base64ImageRaw from "./base64_image.txt?raw";

const base64Image = base64ImageRaw.replace("\n", "").trim();

const verifiedIds = verifiedIdsStr
  .trim()
  .split("\n")
  .reduce((prev, curr) => {
    const [twitter, tz1] = curr.trim().split(",");
    prev[tz1] = twitter;
    return prev;
  }, {});

console.log("verifed ids", verifiedIds);

const tz1Data = tz1DataStr.trim().split("\n");

const tz1Els: HTMLElement[] = [];
for (const tz1 of tz1Data) {
  const twitterHandle = tz1 in verifiedIds ? verifiedIds[tz1] : "anon";
  const el = document.createElement("p");
  el.innerHTML =
    twitterHandle === "anon"
      ? `anon as <i>${tz1}</i>`
      : `@${twitterHandle} as <i>${tz1}</i>`;
  el.style.display = "none";
  document.getElementById("player-popup").appendChild(el);
  tz1Els.push(el);
}
// Get the current URL
const url = new URL(window.location.href);

// Get the value of the 'address' query parameter
const params = new URLSearchParams(url.search);
const targetAddress = params.get("address");
console.log("target address is:", targetAddress);
const targetIndex = tz1Data.indexOf(targetAddress);

const none = document.getElementById("none");
const pixelPlayerMap = new Map();
let currentTimeout;
function revealPlayerByPixel({ x, y }) {
  if (currentTimeout) {
    clearTimeout(currentTimeout);
  }
  currentTimeout = setTimeout(() => {
    for (const el of tz1Els) {
      el.style.display = "none";
    }
    const player = pixelPlayerMap.get(`${x},${y}`);
    if (player) {
      const el = tz1Els[player];
      none.style.display = "none";
      el.style.display = "block";
    } else {
      none.style.display = "block";
    }
  }, 10);
}

function base64ToUint8Array(base64String) {
  const binaryString = window.atob(base64String);
  const uint8Array = new Uint8Array(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    uint8Array[i] = binaryString.charCodeAt(i);
  }
  return uint8Array;
}

const imageData: Uint8Array = base64ToUint8Array(data);

function u32ToNumber(byteArray) {
  return (
    (byteArray[0] << 24) |
    (byteArray[1] << 16) |
    (byteArray[2] << 8) |
    byteArray[3]
  );
}

function loadBaseImage(glWindow) {
  let loadingp = document.querySelector("#loading-p");
  let uiwrapper = document.querySelector("#ui-wrapper");
  loadingp.innerHTML = "downloading map";

  let img = new Image();
  img.src = `data:image/png;base64,${base64Image}`;

  return new Promise<void>((resolve, reject) => {
    img.onload = () => {
      glWindow.setTexture(img);
      glWindow.draw();
      loadingp.innerHTML = "";
      uiwrapper.setAttribute("hide", "true");
      resolve();
    };
    img.onerror = (err) => {
      console.error("Error downloading map.", err);
      reject(err);
    };
  });
}

function decideColor(tz1, color: Uint8Array) {
  if (targetIndex === -1 || targetIndex === tz1) {
    return color;
  }
  const red = color[0];
  const blue = color[1];
  const green = color[2];
  /* remember: if you multiply a number by a decimal between 0
  and 1, it will make the number smaller. That's why we don't
  need to divide the result by three - unlike the previous
  example - because it's already balanced. */

  const r = red * 0.3; // ------> Red is low
  const g = green * 0.59; // ---> Green is high
  const b = blue * 0.11; // ----> Blue is very low

  const gray = r + g + b;

  color[0] = gray;
  color[1] = gray;
  color[2] = gray;

  return color;
}

const sleep = () =>
  new Promise((res, rej) => {
    setTimeout(res, 0);
  });

let animationPaused = false;

async function main() {
  let cvs = document.querySelector("#viewport-canvas") as HTMLCanvasElement;
  let glWindow = new GLWindow(cvs);

  if (!glWindow.ok()) return;

  await loadBaseImage(glWindow);

  let gui = GUI(cvs, glWindow);

  let index = 0;
  const takeU32 = () => {
    const u32 = imageData.subarray(index, index + 4);
    index += 4;
    return u32ToNumber(u32);
  };

  const takeRGBA = () => {
    const rgba = imageData.subarray(index, index + 4);
    index += 4;
    return rgba;
  };

  // const numTz1s = takeU32();
  const numActions = imageData.length / 12;
  // const tz1Map = new Map();
  // for (let i = 0; i < numTz1s; i++) {
  //   tz1Map.set(i, takeTz1());
  // }
  // account for offset
  const batchSize = 10;
  for (let i = 0; i < numActions; i += batchSize) {
    if (animationPaused) {
      while (animationPaused) {
        await sleep();
      }
    }
    for (let j = 0; j < batchSize; j++) {
      const x = takeU32();
      const y = takeU32();
      const rawColor = takeRGBA();
      const tz1Index = takeU32();

      pixelPlayerMap.set(`${x},${y}`, tz1Index);

      const color = decideColor(tz1Index, rawColor);
      glWindow.setPixelColor(x, y, color);
    }
    glWindow.draw();
    if (i % (10 * batchSize) == 0) {
      await sleep();
    }
  }
}

const GUI = (cvs, glWindow) => {
  let color = new Uint8Array([0, 0, 0]);
  let dragdown = false;
  let touchID = 0;
  let touchScaling = false;
  let lastMovePos = { x: 0, y: 0 };
  let lastScalingDist = 0;
  let touchstartTime;

  const colorField = document.querySelector("#color-field");
  const colorSwatch = document.querySelector("#color-swatch");

  // ***************************************************
  // ***************************************************
  // Event Listeners
  //
  document.addEventListener("keydown", (ev) => {
    switch (ev.keyCode) {
      case 189:
      case 173:
        ev.preventDefault();
        zoomOut(1.2);
        break;
      case 187:
      case 61:
        ev.preventDefault();
        zoomIn(1.2);
        break;
    }
  });

  document.addEventListener("keypress", (ev) => {
    switch (ev.keyCode) {
      case 32:
        animationPaused = !animationPaused;
        break;
    }
  });

  window.addEventListener(
    "wheel",
    (ev) => {
      ev.preventDefault();
      let zoom = glWindow.getZoom();
      if (ev.deltaY > 0) {
        zoom /= 1.05;
      } else {
        zoom *= 1.05;
      }
      glWindow.setZoom(zoom);
      glWindow.draw();
    },
    { passive: false }
  );

  document.querySelector("#zoom-in").addEventListener("click", () => {
    zoomIn(1.2);
  });

  document.querySelector("#zoom-out").addEventListener("click", () => {
    zoomOut(1.2);
  });

  window.addEventListener("resize", (ev) => {
    glWindow.updateViewScale();
    glWindow.draw();
  });

  document.addEventListener("mouseup", (ev) => {
    dragdown = false;
    document.body.style.cursor = "auto";
  });

  document.addEventListener("mousemove", (ev) => {
    const movePos = { x: ev.clientX, y: ev.clientY };
    const pos = glWindow.click({ x: movePos.x, y: movePos.y });
    revealPlayerByPixel({ x: Math.floor(pos.x), y: Math.floor(pos.y) });

    if (dragdown) {
      glWindow.move(movePos.x - lastMovePos.x, movePos.y - lastMovePos.y);
      glWindow.draw();
      document.body.style.cursor = "grab";
    }
    lastMovePos = movePos;
  });

  cvs.addEventListener("touchstart", (ev) => {
    let thisTouch = touchID;
    touchstartTime = new Date().getTime();
    lastMovePos = { x: ev.touches[0].clientX, y: ev.touches[0].clientY };
    if (ev.touches.length === 2) {
      touchScaling = true;
      lastScalingDist = null;
    }

    setTimeout(() => {
      if (thisTouch == touchID) {
        animationPaused = !animationPaused;
      }
    }, 350);
  });
  cvs.addEventListener("mousedown", (ev) => {
    switch (ev.button) {
      case 0:
        dragdown = true;
        lastMovePos = { x: ev.clientX, y: ev.clientY };
        break;
    }
  });

  document.addEventListener("touchmove", (ev) => {
    touchID++;
    if (touchScaling) {
      let dist = Math.hypot(
        ev.touches[0].pageX - ev.touches[1].pageX,
        ev.touches[0].pageY - ev.touches[1].pageY
      );
      if (lastScalingDist != null) {
        let delta = lastScalingDist - dist;
        if (delta < 0) {
          zoomIn(1 + Math.abs(delta) * 0.003);
        } else {
          zoomOut(1 + Math.abs(delta) * 0.003);
        }
      }
      lastScalingDist = dist;
    } else {
      let movePos = { x: ev.touches[0].clientX, y: ev.touches[0].clientY };
      glWindow.move(movePos.x - lastMovePos.x, movePos.y - lastMovePos.y);
      glWindow.draw();
      lastMovePos = movePos;
    }
  });

  cvs.addEventListener("contextmenu", () => {
    return false;
  });

  // ***************************************************
  // ***************************************************
  // Helper Functions
  //
  const pickColor = (pos) => {
    color = glWindow.getColor(glWindow.click(pos));
    let hex = "#";
    for (let i = 0; i < color.length; i++) {
      let d = color[i].toString(16);
      if (d.length == 1) d = "0" + d;
      hex += d;
    }
    (colorField as any).value = hex.toUpperCase();
    (colorSwatch as any).style.backgroundColor = hex;
  };

  const zoomIn = (factor) => {
    let zoom = glWindow.getZoom();
    glWindow.setZoom(zoom * factor);
    glWindow.draw();
  };

  const zoomOut = (factor) => {
    let zoom = glWindow.getZoom();
    glWindow.setZoom(zoom / factor);
    glWindow.draw();
  };
};

main();
