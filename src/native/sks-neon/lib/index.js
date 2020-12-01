var addon = require('../native');

console.log(addon.hello());

module.exports.hello = addon.hello;
module.exports.decode = addon.decode;

let MOUSE_KEY_TO_STR = ['left', null, 'right']; // 1 is middle but rust can't handle processing that right now

module.exports.LevelBuilder = class LevelBuilder {
    constructor(board) {
        this.internal = new addon.LevelBuilder();
        
        this.board = board;
        this.boardCtx = this.board.getContext('2d');
        this.boardCtx.imageSmoothingEnabled = true;
        this.dirty = true;

        let mouseHandler = (event) => {
            let {
                x,
                y
            } = this.extractCanvasCoords(event);
            this.internal.updateMousePosition(x, y);

            if (event.type === "mousemove") {}
            else if (event.type === "mousedown") {
                let mouseButton = MOUSE_KEY_TO_STR[event.button];
                if (mouseButton)
                    this.internal.emitMouseButtonEvent(mouseButton, "down");
            } else if (event.type === "mouseup") {
                let mouseButton = MOUSE_KEY_TO_STR[event.button];
                if (mouseButton)
                    this.internal.emitMouseButtonEvent(mouseButton, "up");
            } else {
                console.warn("Unknown Event", event)
            }

            // We are forced to redraw here since we don't know if the events caused a redraw on the rust side.
            // In the future maybe rust could expose the dirty flag? However we would be working in the wrong direction.
            this.dirty = true;
        };

        this.board.addEventListener("mousemove", mouseHandler);

        // Attach globally to capture events outside of canvas
        document.addEventListener("mousedown", mouseHandler);
        document.addEventListener("mouseup", mouseHandler);
        document.addEventListener("keypress", (event) => {
            this.internal.emitRecievedChar(event.key);
            // Same note as above
            this.dirty = true;
        });

        window.addEventListener('keydown', (event) => {
            this.internal.emitKeyboardEvent('down', event.keyCode);
            // Same note as above
            this.dirty = true;
        });
    }

    extractCanvasCoords(event) {
        const rect = this.board.getBoundingClientRect();
        const xRaw = event.clientX - rect.left;
        const yRaw = event.clientY - rect.top;

        const scaleX = this.board.width / rect.width;
        const scaleY = this.board.height / rect.height;

        const x = xRaw * scaleX;
        const y = yRaw * scaleY;
        return {
            x,
            y
        };
    }

    update() {
        this.internal.update();
    }

    enableGrid() {
        this.internal.setGrid(true);
        this.dirty = true;
    }

    disableGrid() {
        this.internal.setGrid(false);
        this.dirty = true;
    }

    isDirty() {
        return this.dirty;
    }

    getImage() {
        return this.internal.getImage();
    }

    // Canvas MUST be 1920 x 1080
    drawFrame(ctx) {
        this.boardCtx.clearRect(0, 0, this.board.width, this.board.height);
        let img = this.internal.getFrame();
        this.boardCtx.drawImage(img, 0, 0, 1920, 1080);
        this.dirty = false;
    }

    export(type) {
        return this.internal.export(type);
    }

    setDark(val) {
        this.internal.setDark(val);
    }

    getDark() {
        return this.internal.getDark();
    }

    import(data) {
        return this.internal.import(data);
        this.dirty = true;
    }

    setLevel(lvl) {
        this.internal.setLevel(lvl);
    }
}
