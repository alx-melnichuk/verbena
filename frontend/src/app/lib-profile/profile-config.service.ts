import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { lastValueFrom } from 'rxjs';

import { Uri } from 'src/app/common/uri';

import { ProfileConfigDto } from './profile-config.interface';

@Injectable({
    providedIn: 'root'
})
export class ProfileConfigService {
    public profileConfigDto: ProfileConfigDto | null = null;

    constructor(private http: HttpClient) { }

    public getConfig(): Promise<ProfileConfigDto | HttpErrorResponse | undefined> {
        if (this.profileConfigDto != null) {
            return Promise.resolve({ ...this.profileConfigDto });
        }
        const url = Uri.appUri('appApi://profiles_config');
        return lastValueFrom(this.http.get<ProfileConfigDto | HttpErrorResponse>(url))
            .then((response: ProfileConfigDto | HttpErrorResponse | undefined) => {
                this.profileConfigDto = response as ProfileConfigDto;
                return { ...this.profileConfigDto };
            });
    }

}
