import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-pg-stream',
  standalone: true,
  imports: [CommonModule, RouterOutlet],
  templateUrl: './pg-stream.component.html',
  styleUrl: './pg-stream.component.scss',
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PgStreamComponent {
  constructor() {
  }
}
  
