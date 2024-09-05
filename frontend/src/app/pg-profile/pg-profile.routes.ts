import { Routes } from '@angular/router';

import { PgProfileComponent } from './pg-profile.component';
import { pgProfileResolver } from './pg-profile.resolver';

export const PG_PROFILE_ROUTES: Routes = [
  {
    path: '',
    component: PgProfileComponent,
    resolve: { profileDto: pgProfileResolver },
  },
];
