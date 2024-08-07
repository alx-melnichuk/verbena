import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { ProfileDto, UniquenessDto } from './profile-api.interface';
import { ProfileApiService } from './profile-api.service';

@Injectable({
  providedIn: 'root'
})
export class ProfileService {
  public profileDto: ProfileDto | null = null;

  constructor(private profileApiService: ProfileApiService) {
  }

  public setProfileDto(profileDto: ProfileDto | null = null): void {
    this.profileDto = profileDto;
  }

  public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
    return this.profileApiService.uniqueness(nickname || '', email || '');
  }

  public async getCurrentProfile(): Promise<ProfileDto | HttpErrorResponse | undefined> {
    const profileDto: ProfileDto = (await this.profileApiService.currentProfile()) as ProfileDto;
    this.profileDto = { ...profileDto } as ProfileDto;
    return Promise.resolve(profileDto);
  }

}
