class Steamworks{
	constructor(){
		this.greenworks = require('greenworks');
		if(this.grenworks.initAPI()){
			this.available = true;
			console.log('Steam API Initialized');
		}else{
			this.available = false;
			console.log('Steam API not Initialized');
		}
	}
}