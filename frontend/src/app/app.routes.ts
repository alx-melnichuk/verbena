import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: 'login',
    loadComponent: () => import('./pg-login/pg-login.component').then((c) => c.PgLoginComponent),
  },
  {
    path: 'signup',
    loadComponent: () => import('./pg-signup/pg-signup.component').then((c) => c.PgSignupComponent),
  },
  {
    path: '',
    loadComponent: () => import('./pg-view/pg-view.component').then((c) => c.PgViewComponent),
  },
  { path: '**', redirectTo: '' },
  //   { path: '', redirectTo: 'login', pathMatch: 'full' },
];
