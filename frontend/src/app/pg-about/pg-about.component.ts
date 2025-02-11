import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

import { PanelAboutComponent } from '../lib-about/panel-about/panel-about.component';

@Component({
    selector: 'app-pg-about',
    standalone: true,
    imports: [CommonModule, PanelAboutComponent],
    templateUrl: './pg-about.component.html',
    styleUrl: './pg-about.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgAboutComponent {

    constructor() {
    }

    // ** Public API **

    // ** Private API **

}
