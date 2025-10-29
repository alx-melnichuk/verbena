import { Routes } from '@angular/router';
import { PgBannedComponent } from './pg-banned.component';
import { PgBannedViewComponent } from './pg-banned-view/pg-banned-view.component';

export const PG_BANNED_ROUTES: Routes = [
    {
        path: '',
        component: PgBannedComponent,
        children: [
            {
                path: '', // 'ind/banned'
                component: PgBannedViewComponent,
            },
        ]
    },
];
