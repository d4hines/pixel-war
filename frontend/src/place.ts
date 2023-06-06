import { blake2bHex, blake2b } from "blakejs";
import { TezosToolkit } from "@taquito/taquito";

globalThis.blake2bHex = blake2bHex
globalThis.blake2b = blake2b;

interface Signer {
  sign: (bytes: string) => Promise<{
    bytes: string;
    sig: string;
    prefixSig: string;
    sbytes: string;
  }>;
  publicKey: () => Promise<string>;
  publicKeyHash: () => Promise<string>;
}

export class Place {
  #loaded;
  #socket;
  #loadingp;
  #uiwrapper;
  #glWindow;
  #tezos;
  #signer;

  constructor(glWindow, tezos: TezosToolkit, signer: Signer) {
    this.#loaded = false;
    this.#socket = null;
    this.#loadingp = document.querySelector("#loading-p");
    this.#uiwrapper = document.querySelector("#ui-wrapper");
    this.#glWindow = glWindow;
    this.#tezos = tezos;
    this.#signer = signer;
  }

  initConnection() {
    this.#loadingp.innerHTML = "connecting";

    let host = window.location.hostname;
    let port = window.location.port;
    if (port != "") {
      host += ":" + port;
    }

    let wsProt;
    if (window.location.protocol == "https:") {
      wsProt = "wss:";
    } else {
      wsProt = "ws:";
    }

    this.#connect(wsProt + "//" + host + "/ws");
    this.#loadingp.innerHTML = "downloading map";

    fetch(window.location.protocol + "//" + host + "/place.png").then(
      async (resp) => {
        if (!resp.ok) {
          console.error("Error downloading map.");
          return null;
        }

        let buf = await this.#downloadProgress(resp);
        await this.#setImage(buf);

        this.#loaded = true;
        this.#loadingp.innerHTML = "";
        this.#uiwrapper.setAttribute("hide", true);
      }
    );
  }

  async #downloadProgress(resp) {
    let len = resp.headers.get("Content-Length");
    let a = new Uint8Array(len);
    let pos = 0;
    let reader = resp.body.getReader();
    while (true) {
      let { done, value } = await reader.read();
      if (value) {
        a.set(value, pos);
        pos += value.length;
        this.#loadingp.innerHTML =
          "downloading map " + Math.round((pos / len) * 100) + "%";
      }
      if (done) break;
    }
    return a;
  }

  #connect(path) {
    this.#socket = new WebSocket(path);
    console.log("connected");

    const socketMessage = async (event) => {
      let data = JSON.parse(event.data);
      this.#handleSocketSetPixel(data.inner.content.PlacePixel);
    };

    const socketClose = (event) => {
      this.#socket = null;
    };

    const socketError = (event) => {
      console.error("Error making WebSocket connection.");
      alert("Failed to connect.");
      this.#socket.close();
    };

    this.#socket.addEventListener("open", () => {console.log("connected")});
    this.#socket.addEventListener("message", socketMessage);
    this.#socket.addEventListener("close", socketClose);
    this.#socket.addEventListener("error", socketError);
  }

  async setPixel(x, y, color) {
    if (this.#socket != null && this.#socket.readyState == 1) {
      console.log("placing:", x, y, color);
      const x_floor = Math.floor(x);
      const y_floor = Math.floor(y);
      const color_values = Object.values(color);
      const inner = {
        nonce: 777,
        content: {
          PlacePixel: {
            x: x_floor,
            y: y_floor,
            color: color_values,
          },
        },
      };
      const hash = blake2bHex(JSON.stringify(inner), undefined, 32);
      const publicKey = await this.#signer.publicKey();
      const { prefixSig } = await this.#signer.sign(hash);
      const message = {
        pkey: {
          Ed25519: publicKey,
        },
        signature: {
          Ed25519: prefixSig,
        },
        inner,
      };
      console.log("=========== message:",message);
      this.#socket.send(JSON.stringify(message));
      this.#glWindow.setPixelColor(x, y, color);
      this.#glWindow.draw();
    } else {
      alert("Disconnected. Probably the server is upgrading. Try refreshing in a few seconds.");
      console.error("Disconnected.");
    }
  }

  #handleSocketSetPixel({ x, y, color }) {
    // if (this.#loaded) {
      this.#glWindow.setPixelColor(x, y, color);
      this.#glWindow.draw();
  }
  

  async #setImage(data) {
    let img = new Image();
    let blob = new Blob([data], { type: "image/png" });
    let blobUrl = URL.createObjectURL(blob);
    img.src = blobUrl;
    let promise = new Promise<void>((resolve, reject) => {
      img.onload = () => {
        this.#glWindow.setTexture(img);
        this.#glWindow.draw();
        resolve();
      };
      img.onerror = reject;
    });
    await promise;
  }

  #putUint32(b, offset, n) {
    let view = new DataView(b);
    view.setUint32(offset, n, false);
  }

  #getUint32(b, offset) {
    let view = new DataView(b);
    return view.getUint32(offset, false);
  }
}

