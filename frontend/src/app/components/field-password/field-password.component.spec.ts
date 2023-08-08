import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldPasswordComponent } from './field-password.component';

describe('FieldPasswordComponent', () => {
  let component: FieldPasswordComponent;
  let fixture: ComponentFixture<FieldPasswordComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldPasswordComponent],
    });
    fixture = TestBed.createComponent(FieldPasswordComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
