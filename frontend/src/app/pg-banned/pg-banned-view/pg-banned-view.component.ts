import { CommonModule } from '@angular/common';
import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { PanelBannedUsersComponent } from 'src/app/lib-banned/panel-banned-users/panel-banned-users.component';

@Component({
    selector: 'app-pg-banned-view',
    standalone: true,
    imports: [CommonModule, PanelBannedUsersComponent],
    templateUrl: './pg-banned-view.component.html',
    styleUrl: './pg-banned-view.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PgBannedViewComponent {

}
