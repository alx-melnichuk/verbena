import { ComponentFixture, TestBed } from '@angular/core/testing';

import { FieldTimeComponent } from './field-time.component';

describe('FieldTimeComponent', () => {
  let component: FieldTimeComponent;
  let fixture: ComponentFixture<FieldTimeComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [FieldTimeComponent]
    });
    fixture = TestBed.createComponent(FieldTimeComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
