import { Routes } from '@angular/router';
import { PgBannedComponent } from './pg-banned.component';
import { pgBannedResolver } from './pg-banned.resolver';
import { BannedUsersComponent } from '../lib-banned/banned-users/banned-users.component';

export const PG_BANNED_ROUTES: Routes = [
    {
        path: '', // 'ind/banned'
        component: PgBannedComponent,
        children: [
            {
                path: '', // 'ind/banned'
                component: BannedUsersComponent,
                resolve: { blockedUsers: pgBannedResolver },
            }
        ]
    },
];
