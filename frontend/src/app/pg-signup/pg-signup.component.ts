import {
  ChangeDetectionStrategy,
  Component,
  ViewEncapsulation,
} from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pg-signup',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './pg-signup.component.html',
  styleUrls: ['./pg-signup.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgSignupComponent {}
