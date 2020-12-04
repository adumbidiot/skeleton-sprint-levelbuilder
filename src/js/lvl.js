// Init rustlib
window.sks = require('sks-neon');

// Browser Code
window.lvl = function (name) {
    this.name = name;
    var self = this;

    this.board = document.createElement('canvas');
    this.board.contentEditable = true;
    this.board.id = this.name;
    this.board.width = 1920;
    this.board.height = 1080;
    this.board.style.cssText = 'width: 80%; border: 0px; outline: 0px;'; // Stay at 80% until we can move the bottom bar into rust
    this.board.style.cursor = 'pointer';

    this.levelBuilder = new window.sks.LevelBuilder(this.board);
    console.log(this.levelBuilder);

    let loopFunc = function () {
        if (self.levelBuilder.isDirty()) {
            let start = performance.now();
            self.levelBuilder.drawFrame();
            let end = performance.now();
            console.log("Dirty Redraw: ", end - start);
        }

        self.levelBuilder.update();
        requestAnimationFrame(loopFunc);
    }

    loopFunc();
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

/*
function checkCtrlZ() {
if (window.lvl.zDown == true && window.lvl.ctrlDown == true && history.length > 0) {
var undo = level.history[level.history.length - 1];
console.log(undo);
level.render(undo.index, undo.oldBlock);
level.history.shift();
}
}
 */
// window.lvl.ctrlDown = false;
// window.lvl.zDown = false;
/*
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
 */

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
