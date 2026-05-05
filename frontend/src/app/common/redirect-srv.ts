import { Injectable } from '@angular/core';
import { environment } from '../../environments/environment';

@Injectable({
    providedIn: 'root',
})
export class RedirectSrv {
    private innUrlAfterLogin: string = "";

    constructor() {
        if (environment.logLevel > 0) { console.info(`RedirectSrv(); // service`); }
    }

    public getUrlAfterLogin(): string {
        return this.innUrlAfterLogin.concat("");
    }
    public setUrlAfterLogin(value: string) {
        this.innUrlAfterLogin = value;
    }

}
