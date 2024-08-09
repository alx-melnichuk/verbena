import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { LoginProfileResponseDto, ProfileDto, ProfileTokensDto, UniquenessDto } from './profile-api.interface';
import { ProfileApiService } from './profile-api.service';

export const ACCESS_TOKEN = 'accessToken';
export const REFRESH_TOKEN = 'refreshToken';

@Injectable({
  providedIn: 'root'
})
export class ProfileService {
  public profileDto: ProfileDto | null = null;
  public profileTokensDto: ProfileTokensDto | null = null;

  constructor(private profileApiService: ProfileApiService) {
  }

  public setProfileDto(profileDto: ProfileDto | null = null): void {
    this.profileDto = profileDto;
  }
  public setProfileTokensDto(profileTokensDto: ProfileTokensDto | null = null): void {
    this.profileTokensDto = this.setProfileTokensDtoToLocalStorage(profileTokensDto);
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
      console.log(`login() this.profileDto: `, this.profileDto);
      return profileResponseDto;
    });
  }

  public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
    return this.profileApiService.uniqueness(nickname || '', email || '');
  }

  public async getCurrentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const profileDto: ProfileDto = (await this.profileApiService.currentProfile()) as ProfileDto;
    this.profileDto = { ...profileDto } as ProfileDto;
    return Promise.resolve(profileDto);
  }

  public delete_profile_current(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    return this.profileApiService.delete_profile_current();
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
