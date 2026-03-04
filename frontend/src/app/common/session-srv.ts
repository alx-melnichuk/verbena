import { Injectable } from '@angular/core';
import { environment } from '../../environments/environment';


export interface User {
    id: number;
    nickname: string; // max_len: 255
    email: string; // max_len: 255
    avatar: string | undefined | null; // Link to user avatar, optional, min_len=2 max_len=255
    theme: string; // Default color theme. ["light","dark"], min_len=2 max_len=32
    locale: string; // Default locale. ["default"], min_len=2 max_len=32
}

export const ACCESS_TOKEN = "accessToken";
export const REFRESH_TOKEN = "refreshToken";

export class UserUtil {
    public static new(user?: Partial<User>): User {
        return {
            id: (user?.id || -1),
            nickname: (user?.nickname || ""),
            email: (user?.email || ""),
            avatar: (user?.avatar || ""),
            theme: (user?.theme || ""),
            locale: (user?.locale || ""),
        };
    }
}

@Injectable({
    providedIn: 'root',
})
export class SessionSrv {
    private accessToken: string | null = null;
    private refreshToken: string | null = null;
    private user: User | null = null;

    constructor() {
        if (environment.logLevel > 0) { console.log(`SessionSrv(); // 2 service`); }
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
        this.user = UserUtil.new(user);
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
}
