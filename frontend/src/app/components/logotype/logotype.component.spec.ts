import { ComponentFixture, TestBed } from '@angular/core/testing';

import { LogotypeComponent } from './logotype.component';

describe('LogotypeComponent', () => {
  let component: LogotypeComponent;
  let fixture: ComponentFixture<LogotypeComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [LogotypeComponent]
    });
    fixture = TestBed.createComponent(LogotypeComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
