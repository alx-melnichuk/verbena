import { ComponentFixture, TestBed } from '@angular/core/testing';

import { UniquenessCheck } from './uniqueness-check';

describe('UniquenessCheck', () => {
  let component: UniquenessCheck;
  let fixture: ComponentFixture<UniquenessCheck>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [UniquenessCheck]
    })
    .compileComponents();

    fixture = TestBed.createComponent(UniquenessCheck);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
