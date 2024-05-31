import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

import { AboutComponent } from '../lib-about/about/about.component';

@Component({
  selector: 'app-pg-about',
  standalone: true,
  imports: [CommonModule, AboutComponent],
  templateUrl: './pg-about.component.html',
  styleUrls: ['./pg-about.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgAboutComponent {

  constructor(
  ) {
    console.log('PgAboutComponent();');
  }

  // ** Public API **

  // ** Private API **

}
