import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ThemePalette } from '@angular/material/core';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

/**
 * <app-spinner *ngIf="isLoad" [isFullOwner]="true"></app-spinner>
 */

const DEAULT_DIAMETER = 100;

@Component({
  selector: 'app-spinner',
  standalone: true,
  imports: [CommonModule, MatProgressSpinnerModule],
  templateUrl: './spinner.component.html',
  styleUrls: ['./spinner.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class SpinnerComponent {
  @Input()
  public color: ThemePalette = 'primary';
  @Input()
  public isFullOwner = false;
  @Input()
  public isFullscreen = false;
  @Input()
  public diameter = DEAULT_DIAMETER;

  @HostBinding('class.fullscreen')
  public get isFullscreenVal(): boolean {
    return !!this.isFullscreen;
  }

}
