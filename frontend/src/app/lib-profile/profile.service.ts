import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';

import { UniquenessDto } from './profile-api.interface';
import { ProfileApiService } from './profile-api.service';

@Injectable({
  providedIn: 'root'
})
export class ProfileService {

  constructor(private profileApiService: ProfileApiService) {
  }

  public uniqueness(nickname: string, email: string): Promise<UniquenessDto | HttpErrorResponse | undefined> {
    return this.profileApiService.uniqueness(nickname || '', email || '');
  }

}
