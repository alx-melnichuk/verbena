import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';

@Component({
    selector: 'app-avatar',
    exportAs: 'appAvatar',
    standalone: true,
    imports: [],
    templateUrl: './avatar.component.html',
    styleUrl: './avatar.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class AvatarComponent {
    @Input()
    public avatar: string | null | undefined;
}
