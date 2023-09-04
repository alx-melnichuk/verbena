import { HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { UserApiService } from './user-api.service';
import { SendUserRegistrationDto, UserDto } from './user-dto';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  constructor(private userApiService: UserApiService) {}

  public registration(nickname: string, email: string, password: string): Promise<UserDto | HttpErrorResponse | undefined> {
    if (!nickname || !email || !password) {
      return Promise.reject();
    }
    const sendUserRegistrationDto: SendUserRegistrationDto = { nickname, email, password };
    return this.userApiService.addUser(sendUserRegistrationDto);
  }
}
