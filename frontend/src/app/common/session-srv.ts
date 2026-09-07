import { Injectable } from "@angular/core";
import { environment as env } from "../../environments/environment";


export interface User {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    theme: string; // Default color theme. ["light","dark"], min_len=2 max_len=32
    locale: string; // Default locale. ["default"], min_len=2 max_len=32
}

export interface Settings {
    appearance?: string | undefined;
    formFieldShape?: string | undefined;
    buttonShape?: string | undefined;
}

export const ACCESS_TOKEN = "accessToken";
export const REFRESH_TOKEN = "refreshToken";

@Injectable({
    providedIn: "root",
})
export class SessionSrv {
    private accessToken: string | null = null;
    private refreshToken: string | null = null;
    private user: User | null = null;
    private settings: Settings | null = null;

    constructor() {
        if (env.logLevel & 4) { console.info(`SessionSrv(); // 2 service`); }
        const accessToken = localStorage.getItem(ACCESS_TOKEN);
        const refreshToken = localStorage.getItem(REFRESH_TOKEN);
        if (!!accessToken && !!refreshToken) {
            this.setUserTokens(accessToken, refreshToken);
        }
    }

    public getUser(): User | null {
        return this.user != null ? { ...this.user } : null;
    }
    public setUser(user: Partial<User>): void {
        this.user = {
            id: (user?.id || -1),
            nickname: (user?.nickname || ""),
            email: (user?.email || ""),
            avatar: (user?.avatar || ""),
            theme: (user?.theme || ""),
            locale: (user?.locale || ""),
        };
    }
    public removeUser(): void {
        this.user = null;
    }

    public getAccessToken(): string | null {
        return this.accessToken?.concat("") || null;
    }
    public getRefreshToken(): string | null {
        return this.refreshToken?.concat("") || null;
    }
    public setUserTokens(accessToken: string, refreshToken: string): void {
        this.accessToken = accessToken;
        this.refreshToken = refreshToken;
        window.localStorage.setItem(ACCESS_TOKEN, this.accessToken);
        window.localStorage.setItem(REFRESH_TOKEN, this.refreshToken);
    }
    public removeUserTokens(): void {
        this.accessToken = null;
        this.refreshToken = null;
        window.localStorage.removeItem(ACCESS_TOKEN);
        window.localStorage.removeItem(REFRESH_TOKEN);
    }
    public getSettings(): Settings | null {
        return this.settings != null ? { ...this.settings } : null;
    }
    public setSettings(value: Partial<Settings>) {
        this.settings = {
            appearance: value.appearance,
            formFieldShape: value.formFieldShape,
            buttonShape: value.buttonShape,
        };
        console.log(`SessionSrv().setSettings(${JSON.stringify(this.settings)})`); // #
    }
}
