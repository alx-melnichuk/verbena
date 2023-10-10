import { ComponentFixture, TestBed } from '@angular/core/testing';

import { AlertWrapComponent } from './alert-wrap.component';

describe('AlertWrapComponent', () => {
  let component: AlertWrapComponent;
  let fixture: ComponentFixture<AlertWrapComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [AlertWrapComponent],
    });
    fixture = TestBed.createComponent(AlertWrapComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
