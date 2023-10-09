import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-pg-view',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './pg-view.component.html',
  styleUrls: ['./pg-view.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgViewComponent {
  constructor() {
    console.log(`PgViewComponent()`); // #-
  }
}
