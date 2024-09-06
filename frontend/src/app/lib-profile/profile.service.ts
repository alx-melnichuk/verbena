import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import {
 LoginProfileResponseDto, ModifyProfileDto, NewPasswordProfileDto, ProfileDto, ProfileTokensDto, TokenUpdate, UniquenessDto
} from './profile-api.interface';
import { ProfileApiService } from './profile-api.service';

export const ACCESS_TOKEN = 'accessToken';
export const REFRESH_TOKEN = 'refreshToken';

@Injectable({
  providedIn: 'root'
})
export class ProfileService implements TokenUpdate {
  public profileDto: ProfileDto | null = null;
  public profileTokensDto: ProfileTokensDto | null = null;

  constructor(private profileApiService: ProfileApiService) {
    this.profileTokensDto = this.getProfileTokensDtoFromLocalStorage();
  }

  public setProfileDto(profileDto: ProfileDto | null = null): void {
    this.profileDto = profileDto;
  }
  public setProfileTokensDto(profileTokensDto: ProfileTokensDto | null = null): void {
    this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(profileTokensDto);
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

  public login(nickname: string, password: string): Promise<LoginProfileResponseDto | HttpErrorResponse | undefined> {
    if (!nickname || !password) {
      return Promise.reject();
    }

    this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(null);
    return this.profileApiService.login({ nickname, password })
    .then((response: LoginProfileResponseDto | HttpErrorResponse | undefined) => {
      let profileResponseDto: LoginProfileResponseDto = response as LoginProfileResponseDto;
      this.profileDto = { ...profileResponseDto.profileDto } as ProfileDto;
      this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(profileResponseDto.profileTokensDto);
      return profileResponseDto;
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
        this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(null);
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
  public refreshToken(): Promise<ProfileTokensDto | HttpErrorResponse> {
    if (!this.profileTokensDto?.refreshToken) {
      return Promise.reject();
    }
    return this.profileApiService
      .refreshToken({ token: this.profileTokensDto.refreshToken })
      .then((response: HttpErrorResponse | ProfileTokensDto | undefined) => {
        this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(response as ProfileTokensDto);
        return response as ProfileTokensDto;
      })
      .catch((error) => {
        // Remove "Token" values in LocalStorage.
        this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(null);
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
  private setProfileTokensDtoToLocalStorage(profileTokensDto: ProfileTokensDto | null): ProfileTokensDto | null {
    const accessToken = profileTokensDto?.accessToken || null;
    this.updateItemInLocalStorage(ACCESS_TOKEN, accessToken);
    const refreshToken = profileTokensDto?.refreshToken || null;
    this.updateItemInLocalStorage(REFRESH_TOKEN, refreshToken);
    return !!profileTokensDto ? { ...profileTokensDto } : null;
  }
  private getProfileTokensDtoFromLocalStorage(): ProfileTokensDto | null {
    let result: ProfileTokensDto | null = null;
    const accessToken = localStorage.getItem(ACCESS_TOKEN);
    const refreshToken = localStorage.getItem(REFRESH_TOKEN);
    if (!!accessToken && !!refreshToken) {
      result = { accessToken, refreshToken };
    }
    return result;
  }

}
