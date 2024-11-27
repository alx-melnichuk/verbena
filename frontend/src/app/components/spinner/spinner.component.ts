import { ChangeDetectionStrategy, Component, HostBinding, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ThemePalette } from '@angular/material/core';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

/**
 * // Simple display.
 * <app-spinner></app-spinner>
 * 
 * // Fill the space of the parent element (which should have: "position: relative;").
 * <div style="position: relative; height: 300px; width: 300px;">
 *   <app-spinner isFillParent></app-spinner>
 * <div>
 */

const DEAULT_DIAMETER = 100;

@Component({
  selector: 'app-spinner',
  exportAs: 'appSpinner',
  standalone: true,
  imports: [CommonModule, MatProgressSpinnerModule],
  templateUrl: './spinner.component.html',
  styleUrl: './spinner.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class SpinnerComponent {
  @Input()
  public color: ThemePalette = 'primary';
  @Input()
  public isFillParent: string | null | undefined;
  @Input()
  public diameter = DEAULT_DIAMETER;

  @HostBinding('class.fill-parent')
  public get isClassFillParent(): boolean {
    return !!(this.isFillParent == '' || this.isFillParent == 'true');
  }
}
