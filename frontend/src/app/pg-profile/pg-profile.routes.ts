import { Routes } from '@angular/router';

import { PgProfileComponent } from './pg-profile.component';
import { pgUserInfoResolver } from './pg-user-info.resolver';

export const PG_PROFILE_ROUTES: Routes = [
  {
    path: '',
    component: PgProfileComponent,
    resolve: { userDto: pgUserInfoResolver },
  },
];
