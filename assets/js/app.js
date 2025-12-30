function app() {
  return {
    isMain:
      (typeof window !== "undefined" && !window.__TAURI_WINDOW__) ||
      window.__TAURI_WINDOW__.label === "main",

    async init() {
      if (!window.__TAURI__) {
        console.error("Tauri API not available");
        return;
      }

      const currentWindow = await window.__TAURI__.window.getCurrent();
      this.isMain = currentWindow.label === "main";

      if (this.isMain) {
        this.initMain();
      } else {
        this.initOverlay();
      }
    },

    initMain() {
      document.addEventListener("keydown", (event) => {
        if (event.metaKey && event.key === "s") {
          event.preventDefault();
          this.startSearch();
        }
      });
    },

    initOverlay() {
      document.addEventListener("keydown", (event) => {
        if (event.key === "Escape") {
          this.cancelSearch();
        }
      });

      this.loadScreenshot();
    },

    async startSearch() {
      try {
        await window.__TAURI__.core.invoke("show_overlay_window");
      } catch (error) {
        console.error("Failed to start search:", error);
      }
    },

    async cancelSearch() {
      try {
        await window.__TAURI__.core.invoke("hide_overlay_window");
      } catch (error) {
        console.error("Failed to hide overlay:", error);
      }
    },

    async loadScreenshot() {
      try {
        const response = await window.__TAURI__.core.invoke(
          "capture_screenshot"
        );
        this.screenshot = response;
        this.$nextTick(() => {
          this.drawScreenshot();
        });
      } catch (error) {
        console.error("Failed to capture screenshot:", error);
        this.cancelSearch();
      }
    },

    startSelection(event) {
      if (!this.isMain) {
        this.selection = this.selection || {};
        this.selection.isDrawing = true;
        this.selection.startX = event.offsetX;
        this.selection.startY = event.offsetY;
      }
    },

    updateSelection(event) {
      if (this.isMain || !this.selection || !this.selection.isDrawing) return;
      this.selection.endX = event.offsetX;
      this.selection.endY = event.offsetY;
      this.drawScreenshot();
    },

    endSelection() {
      if (this.isMain || !this.selection) return;
      this.selection.isDrawing = false;
    },

    drawScreenshot() {
      const canvas = this.$refs?.canvas;
      if (!canvas || !this.screenshot) return;

      const ctx = canvas.getContext("2d");
      if (!ctx) return;

      canvas.width = this.screenshot.width;
      canvas.height = this.screenshot.height;

      const img = new Image();
      const blob = new Blob([new Uint8Array(this.screenshot.image_data)], {
        type: "image/png",
      });
      const url = URL.createObjectURL(blob);

      img.onload = () => {
        ctx.drawImage(img, 0, 0);
        if (this.selection?.isDrawing) {
          this.drawSelection(ctx);
        }
        URL.revokeObjectURL(url);
      };

      img.onerror = () => {
        console.error("Failed to load screenshot image");
        URL.revokeObjectURL(url);
      };

      img.src = url;
    },

    drawSelection(ctx) {
      const minX = Math.min(this.selection.startX, this.selection.endX);
      const minY = Math.min(this.selection.startY, this.selection.endY);
      const width = Math.abs(this.selection.endX - this.selection.startX);
      const height = Math.abs(this.selection.endY - this.selection.startY);

      ctx.strokeStyle = "rgba(0, 102, 204, 0.8)";
      ctx.lineWidth = 3;
      ctx.strokeRect(minX, minY, width, height);

      ctx.fillStyle = "rgba(0, 102, 204, 0.1)";
      ctx.fillRect(minX, minY, width, height);
    },

    screenshot: null,
    selection: { startX: 0, startY: 0, endX: 0, endY: 0, isDrawing: false },
  };
}
