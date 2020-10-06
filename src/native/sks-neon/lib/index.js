var addon = require('../native');

console.log(addon.hello());

let data = ['b0', 'a1'];
console.log(addon.export1DPatch(data));

module.exports.hello = addon.hello;

module.exports.encodeBlockLBL = addon.encodeBlockLBL;

module.exports.decode = addon.decode;

module.exports.LevelBuilder = class LevelBuilder {
    constructor() {
        this.internal = new addon.LevelBuilder();
        this.dirty = true;
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
        this.internal.getImage();
    }

    getLevelData() {
        return this.internal.getLevelData();
    }

    getImage() {
        return this.internal.getImage();
    }

    // Canvas MUST be 1920 x 1080
    drawFrame(ctx) {
        let img = this.internal.getFrame();
        ctx.drawImage(img, 0, 0, 1920, 1080);
        this.dirty = false;
    }

    addBlock(i, block) {
        this.internal.addBlock(i, block);
        this.dirty = true;
    }

    export(type) {
        return this.internal.export(type);
    }

    exportLevel() {
        return this.internal.exportLevel();
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
