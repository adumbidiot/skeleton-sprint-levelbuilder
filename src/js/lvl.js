// Init rustlib
window.sks = require('sks-neon');

// Browser Code
window.lvl = function (name) {
    this.name = name;
    this.active = null;
    var self = this;

    this.board = document.createElement('canvas');
    this.board.id = this.name;
    this.board.width = 1920;
    this.board.height = 1080;
    this.board.style.cssText = 'width: 80%;';
    this.bgCtx = this.board.getContext('2d');

    this.levelBuilder = new window.sks.LevelBuilder();
    console.log(this.levelBuilder);

    let loopFunc = function () {
        if (self.levelBuilder.isDirty()) {
            let start = performance.now();
            self.bgCtx.clearRect(0, 0, self.board.width, self.board.height);
            self.levelBuilder.drawFrame(self.bgCtx);
            let end = performance.now();
            console.log("Dirty Redraw: ", end - start);
        }
        requestAnimationFrame(loopFunc);
    }

    loopFunc();

    let boardHandler = (event) => {
        const blockSize = this.board.width / 32;
        const rect = this.board.getBoundingClientRect();
        const xRaw = event.clientX - rect.left;
        const yRaw = event.clientY - rect.top;

        const scaleX = this.board.width / rect.width;
        const scaleY = this.board.height / rect.height;

        const x = xRaw * scaleX;
        const y = yRaw * scaleY;

        const index = Math.floor(x / blockSize) + (Math.floor(y / blockSize) * 32);

        // Give time for mouse states to update
        setTimeout(() => {
            if (window.lvl.mouseDownRight) {
                this.render(index, this.active);
            }

            if (window.lvl.mouseDownLeft) {
                this.render(index, "delete");
            }
        }, 0);

        event.preventDefault();
    };

    this.board.addEventListener("mousemove", boardHandler);
    this.board.addEventListener("mousedown", boardHandler);
    this.board.addEventListener("mouseup", boardHandler);
}

// Renders a specified block at specified index
window.lvl.prototype.render = function (index, blockType) {
    if (!this.active)
        return;
    if (blockType == 'delete')
        blockType = 'null';
    this.levelBuilder.addBlock(index, blockType);
}

window.lvl.prototype.setDark = function (value) {
    this.levelBuilder.setDark(value);
}
// Disables grid on board
window.lvl.prototype.disableGrid = function () {
    this.levelBuilder.disableGrid();
}
//Enables grid
window.lvl.prototype.enableGrid = function () {
    this.levelBuilder.enableGrid();
}

lvl.prototype.exportLBL = function () {
    return this.levelBuilder.export('lbl');
}

lvl.prototype.exportPNG = function (cb) {
    return this.levelBuilder.getImage().toDataURL('image/png');
}

lvl.prototype.exportDev = function (num) {
    this.levelBuilder.setLevel(num);
    return this.levelBuilder.export('as3');
}

// Guesses the import format and imports
lvl.prototype.import = function (data) {
    let res = this.levelBuilder.import(data);
    document.getElementById('dark').checked = this.levelBuilder.getDark();
    return res;
}

function checkCtrlZ() {
    if (window.lvl.zDown == true && window.lvl.ctrlDown == true && history.length > 0) {
        var undo = level.history[level.history.length - 1];
        console.log(undo);
        level.render(undo.index, undo.oldBlock);
        level.history.shift();
    }
}

window.lvl.mouseDown = false;
window.lvl.mouseDownLeft = false;
window.lvl.mouseDownRight = false;

window.lvl.ctrlDown = false;
window.lvl.zDown = false;
document.onmousedown = function (e) {
    if (e.button == 0) {
        window.lvl.mouseDownRight = true;
    } else if (e.button == 2) {
        window.lvl.mouseDownLeft = true;
    }
    window.lvl.mouseDown = true;
}
document.onmouseup = function (e) {
    if (e.button == 0) {
        window.lvl.mouseDownRight = false;
    } else if (e.button == 2) {
        window.lvl.mouseDownLeft = false;
    }
    window.lvl.mouseDown = false;
}
document.onkeydown = function (event) {
    switch (event.keyCode) {
    case 17: {
            window.lvl.ctrlDown = true;
            break;
        }
    case 90: {
            window.lvl.zDown = true;
            break;
        }
    }
    checkCtrlZ();
}
document.onkeyup = function (event) {
    switch (event.keyCode) {
    case 17: {
            window.lvl.ctrlDown = false;
            break;
        }
    case 90: {
            window.lvl.zDown = false;
            break;
        }
    }
    checkCtrlZ();
}

// Electron stuff
try {
    window.greenworks = require('greenworks');
} catch (e) {
    console.error('Greenworks dll error');
    greenworks = {
        initAPI: function () {
            return false;
        }
    };
}

window.path = require('path');
window.process = require('process');
window.remote = require('electron').remote;
window.fs = require('fs');

window.dialog = remote.dialog;
window.srcDir = process.cwd();

console.log('srcDir: ' + srcDir);
process.activateUvLoop();
window.onerror = function (errorMsg, url, lineNumber) {
    alert(errorMsg + ' line: ' + lineNumber);
};

// Steamworks init
window.steam = false;
if (greenworks.initAPI()) {
    console.log('Steamworks API Initialized');
    window.steam = true;

    let steamworksWorkshopSyncPath = path.resolve(srcDir, 'game/Custom Levels');

    if (!fs.existsSync(steamworksWorkshopSyncPath)) {
        try {
            fs.mkdirSync(steamworksWorkshopSyncPath);
        } catch (e) {
            alert(e);
            console.error(e);
        }
    }
    try {
        greenworks.ugcSynchronizeItems(steamworksWorkshopSyncPath, function (items) {
            console.log('Workshop Items Loaded: ');
            console.log(items);
        }, function (err) {
            throw err;
        });
    } catch (e) {
        alert(e);
    }
} else {
    console.log('Steamworks API Initialization Failed');
}

// Init
window.level = new lvl('build');

// Util

async function getFilename() {
    let filename = await window.dialog.showOpenDialog();
    if (!filename) {
        throw "No Dialog Data";
    }
    let data = await readFile(filename[0], 'utf8');
    return data;
}

function readFile(path, encoding) {
    return new Promise((resolve, reject) => {
        window.fs.readFile(path, encoding, function (err, data) {
            if (err) {
                reject(err);
            } else {
                resolve(data);
            }
        });
    });
}

function openImportPopup() {
    getFilename()
    .then((data) => {
        if (!window.level.import(data)) {
            alert("Failed to load file");
        }
    })
    .catch((e) => {
        throw e;
    });
}
