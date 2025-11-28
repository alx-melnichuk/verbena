import { Routes } from '@angular/router';
import { PgBannedComponent } from './pg-banned.component';
import { pgBannedResolver } from './pg-banned.resolver';

export const PG_BANNED_ROUTES: Routes = [
    {
        path: '', // 'ind/banned'
        component: PgBannedComponent,
        resolve: { blockedUsers: pgBannedResolver },
    },
];
