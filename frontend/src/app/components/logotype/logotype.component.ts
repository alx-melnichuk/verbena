import { ChangeDetectionStrategy, Component, Input, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '@ngx-translate/core';

@Component({
  selector: 'app-logotype',
  exportAs: 'appLogotype',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './logotype.component.html',
  styleUrl: './logotype.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class LogotypeComponent {
  @Input()
  public logo: string | null = null;
}
