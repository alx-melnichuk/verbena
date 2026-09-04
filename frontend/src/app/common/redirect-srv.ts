import { Injectable } from "@angular/core";
import { environment as env } from "../../environments/environment";

@Injectable({
    providedIn: "root",
})
export class RedirectSrv {
    private innUrlAfterLogin: string = "";

    constructor() {
        if (env.logLevel & 4) { console.info(`RedirectSrv(); // 5 - service`); }
    }

    public getUrlAfterLogin(): string {
        return this.innUrlAfterLogin.concat("");
    }
    public setUrlAfterLogin(value: string) {
        this.innUrlAfterLogin = value;
    }

}
