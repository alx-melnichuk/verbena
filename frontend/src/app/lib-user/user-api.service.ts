import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';

import { Uri } from 'src/app/common/uri';
import { CreateUserDto, RecoveryUserDto } from './user-api.interface';

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

}
