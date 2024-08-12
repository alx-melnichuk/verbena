import { HttpClient, HttpErrorResponse, HttpHeaders, HttpParams } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import {
  CreateUserDto, ModifyProfileDto, RecoveryUserDto, UpdatePasswordDto, UserDto, UserProfileDto
} from './user-api.interface';

@Injectable({
  providedIn: 'root',
})
export class UserApiService {
  constructor(private http: HttpClient) {}

  public registration(createUserDto: CreateUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://registration');
    return this.http.post<null | HttpErrorResponse>(url, createUserDto).toPromise();
  }

  public recovery(recoveryUserDto: RecoveryUserDto): Promise<null | HttpErrorResponse | undefined> {
    const url = Uri.appUri('appApi://recovery');
    return this.http.post<null | HttpErrorResponse>(url, recoveryUserDto).toPromise();
  }

  public modifyProfile(
    id: number, modifyProfileDto: ModifyProfileDto, file?: File | null
  ): Promise<UserProfileDto | HttpErrorResponse | undefined> {
    const formData: FormData = new FormData();
    if (modifyProfileDto.nickname != null) {
      formData.set('nickname', modifyProfileDto.nickname);
    }
    if (modifyProfileDto.email != null) {
      formData.set('email', modifyProfileDto.email);
    }
    if (modifyProfileDto.password != null) {
      formData.set('password', modifyProfileDto.password);
    }
    if (modifyProfileDto.descript != null) {
      formData.set('descript', modifyProfileDto.descript);
    }
    if (file !== undefined) {
      const currFile: File = (file !== null ? file : new File([], "file"));
      formData.set('avatarfile', currFile, currFile.name);
    }
    const headers = new HttpHeaders({ 'enctype': 'multipart/form-data' });
    const url = Uri.appUri(`appApi://users/profile/${id}`);
    return this.http.put<UserProfileDto | HttpErrorResponse>(url, formData, { headers: headers }).toPromise();
  }

  public new_password(updatePasswordDto: UpdatePasswordDto): Promise<UserDto | HttpErrorResponse | undefined> {
    if (!updatePasswordDto.password && !updatePasswordDto.new_password) {
      return Promise.resolve(undefined);
    }
    const url = Uri.appUri("appApi://users_new_password");
    return this.http.put<UserDto | HttpErrorResponse>(url, updatePasswordDto).toPromise();
  }
}
