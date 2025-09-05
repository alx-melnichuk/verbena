import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { LocalStorageUtil } from '../utils/local-storage.util';

import {
    LoginResponseDto, ModifyProfileDto, NewPasswordProfileDto, ProfileDto, UserTokenResponseDto, TokenUpdate, UniquenessDto,
    ProfileDtoUtil
} from './profile-api.interface';
import { ProfileApiService } from './profile-api.service';

export const ACCESS_TOKEN = 'accessToken';
export const REFRESH_TOKEN = 'refreshToken';

@Injectable({
    providedIn: 'root'
})
export class ProfileService implements TokenUpdate {
    public profileDto: ProfileDto | null = null;
    public profileTokensDto: UserTokenResponseDto | null = null;

    constructor(private profileApiService: ProfileApiService) {
        this.profileTokensDto = this.getUserTokensDtoFromLocalStorage();
    }

    public getProfileDto(): ProfileDto | null {
        return this.profileDto != null ? { ...this.profileDto } : null;
    }
    public setProfileDto(profileDto: ProfileDto | null = null): void {
        this.profileDto = profileDto;
    }
    public getUserTokensDto(): UserTokenResponseDto | null {
        return this.profileTokensDto != null ? { ...this.profileTokensDto } : null;
    }
    public setUserTokensDto(userTokensDto: UserTokenResponseDto | null = null): void {
        this.profileTokensDto = this.setUserTokensDtoToLocalStorage(userTokensDto);
    }

    public hasAccessTokenInLocalStorage(): boolean {
        return !!localStorage.getItem(ACCESS_TOKEN);
    }

    public registration(nickname: string, email: string, password: string): Promise<null | HttpErrorResponse | undefined> {
        if (!nickname || !email || !password) {
            return Promise.reject();
        }
        return this.profileApiService.registration({ nickname, email, password });
    }

    public recovery(email: string): Promise<null | HttpErrorResponse | undefined> {
        if (!email) {
            return Promise.reject();
        }
        return this.profileApiService.recovery({ email });
    }

    public login(nickname: string, password: string): Promise<LoginResponseDto | HttpErrorResponse | undefined> {
        if (!nickname || !password) {
            return Promise.reject();
        }

        this.profileTokensDto = this.setUserTokensDtoToLocalStorage(null);
        return this.profileApiService.login({ nickname, password })
            .then((response: LoginResponseDto | HttpErrorResponse | undefined) => {
                let loginResponseDto: LoginResponseDto = response as LoginResponseDto;
                this.profileDto = ProfileDtoUtil.new(loginResponseDto.userProfileDto);
                this.profileTokensDto = this.setUserTokensDtoToLocalStorage(loginResponseDto.tokenUserResponseDto);
                return loginResponseDto;
            });
    }

    public logout(): Promise<void | HttpErrorResponse> {
        if (!this.profileTokensDto?.accessToken) {
            return Promise.reject();
        }
        return this.profileApiService.logout()
            .finally(() => {
                // Reset authorization settings even if an error occurs.
                this.profileDto = null;
                this.profileTokensDto = this.setUserTokensDtoToLocalStorage(null);
                return;
            });
    }
    // ** interface TokenUpdate **
    public isCheckRefreshToken(method: string, url: string): boolean {
        return this.profileApiService.isCheckRefreshToken(method, url);
    }
    public isExistRefreshToken(): boolean {
        return !!this.profileTokensDto?.refreshToken;
    }
    public getAccessToken(): string | null {
        return this.profileTokensDto?.accessToken || null;
    }
    public refreshToken(): Promise<UserTokenResponseDto | HttpErrorResponse> {
        if (!this.profileTokensDto?.refreshToken) {
            return Promise.reject();
        }
        return this.profileApiService
            .refreshToken({ token: this.profileTokensDto.refreshToken })
            .then((response: HttpErrorResponse | UserTokenResponseDto | undefined) => {
                this.profileTokensDto = this.setUserTokensDtoToLocalStorage(response as UserTokenResponseDto);
                return response as UserTokenResponseDto;
            })
            .catch((error) => {
                // Remove "Token" values in LocalStorage.
                this.profileTokensDto = this.setUserTokensDtoToLocalStorage(null);
                // Return error.
                throw error;
            });
    }
    // ** **
    public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
        return this.profileApiService.uniqueness(nickname || '', email || '');
    }

    public async getCurrentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
        const profileDto: ProfileDto = (await this.profileApiService.currentProfile()) as ProfileDto;
        this.profileDto = { ...profileDto } as ProfileDto;
        return Promise.resolve(profileDto);
    }

    public modifyProfile(modifyProfileDto: ModifyProfileDto, file?: File | null): Promise<ProfileDto | HttpErrorResponse | undefined> {
        return this.profileApiService.modifyProfile(modifyProfileDto, file);
    }

    public newPassword(newPasswordProfileDto: NewPasswordProfileDto): Promise<ProfileDto | HttpErrorResponse | undefined> {
        return this.profileApiService.newPassword(newPasswordProfileDto);
    }

    public deleteProfileCurrent(): Promise<ProfileDto | HttpErrorResponse | undefined> {
        return this.profileApiService.deleteProfileCurrent();
    }

    // ** Private Api **

    private updateItemInLocalStorage(name: string, value: string | null): void {
        if (!!name) {
            if (!!value) {
                localStorage.setItem(name, value);
            } else {
                localStorage.removeItem(name);
            }
        }
    }
    private setUserTokensDtoToLocalStorage(profileTokensDto: UserTokenResponseDto | null): UserTokenResponseDto | null {
        LocalStorageUtil.update(ACCESS_TOKEN, profileTokensDto?.accessToken || null);
        LocalStorageUtil.update(REFRESH_TOKEN, profileTokensDto?.refreshToken || null);

        return !!profileTokensDto ? { ...profileTokensDto } : null;
    }
    private getUserTokensDtoFromLocalStorage(): UserTokenResponseDto | null {
        let result: UserTokenResponseDto | null = null;
        const accessToken = localStorage.getItem(ACCESS_TOKEN);
        const refreshToken = localStorage.getItem(REFRESH_TOKEN);
        if (!!accessToken && !!refreshToken) {
            result = { accessToken, refreshToken };
        }
        return result;
    }

}
